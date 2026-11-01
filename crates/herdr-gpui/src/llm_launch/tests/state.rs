use super::*;
use Blocker::{Foreign, Row};
use Health::{Down, Loading, Ready};
use RowState::{Blocked, Running, Starting, Stopped};

const MAC: &str = "Local";

fn row(
    host: &'static str,
    group: Option<&'static str>,
    health: Option<Health>,
) -> RowInput<'static> {
    RowInput {
        host,
        group,
        health,
    }
}

fn mac(health: Option<Health>) -> RowInput<'static> {
    row(MAC, Some("mac-llm"), health)
}

#[test]
fn a_ready_row_runs_and_blocks_the_rest_of_its_group() {
    let rows = [mac(Some(Ready)), mac(Some(Down)), mac(Some(Down))];
    assert_eq!(
        row_states(&rows, &[]),
        [Running, Blocked { by: Row(0) }, Blocked { by: Row(0) }]
    );
}

#[test]
fn a_loading_row_is_starting_and_blocks_the_others() {
    let rows = [mac(Some(Loading)), mac(Some(Down)), mac(None)];
    assert_eq!(
        row_states(&rows, &[]),
        [Starting, Blocked { by: Row(0) }, Blocked { by: Row(0) }]
    );
}

#[test]
fn the_slot_holder_need_not_be_first() {
    let rows = [mac(Some(Down)), mac(Some(Ready)), mac(Some(Down))];
    assert_eq!(
        row_states(&rows, &[]),
        [Blocked { by: Row(1) }, Running, Blocked { by: Row(1) }]
    );
}

#[test]
fn two_running_rows_both_stay_running_and_a_third_is_blocked_by_the_first() {
    let rows = [
        mac(Some(Down)),
        mac(Some(Ready)),
        mac(Some(Ready)),
        mac(Some(Down)),
    ];
    assert_eq!(
        row_states(&rows, &[]),
        [
            Blocked { by: Row(1) },
            Running,
            Running,
            Blocked { by: Row(1) }
        ]
    );
}

#[test]
fn a_starting_row_before_a_running_one_is_the_blocker() {
    let rows = [mac(Some(Down)), mac(Some(Loading)), mac(Some(Ready))];
    assert_eq!(
        row_states(&rows, &[]),
        [Blocked { by: Row(1) }, Starting, Running]
    );
}

#[test]
fn nothing_running_leaves_every_row_stopped() {
    let rows = [mac(Some(Down)), mac(None), mac(Some(Down))];
    assert_eq!(row_states(&rows, &[]), [Stopped, Stopped, Stopped]);
}

#[test]
fn ungrouped_rows_are_never_blocked() {
    let rows = [
        row(MAC, None, Some(Ready)),
        row(MAC, None, Some(Down)),
        row(MAC, Some("mac-llm"), Some(Down)),
    ];
    assert_eq!(row_states(&rows, &[]), [Running, Stopped, Stopped]);
}

#[test]
fn hosts_and_groups_never_interact() {
    let rows = [
        mac(Some(Ready)),
        row("remoteluci", Some("mac-llm"), None),
        row(MAC, Some("other"), Some(Down)),
        mac(Some(Down)),
    ];
    assert_eq!(
        row_states(&rows, &[]),
        [Running, Stopped, Stopped, Blocked { by: Row(0) }]
    );
}

#[test]
fn the_output_matches_the_input_in_order_and_length() {
    assert!(row_states(&[], &[]).is_empty());
    let rows = [mac(Some(Down)), mac(Some(Ready))];
    assert_eq!(row_states(&rows, &[]).len(), rows.len());
}

#[test]
fn a_probe_status_maps_to_a_health() {
    for (status, health) in [
        (Some(200), Ready),
        (Some(503), Loading),
        (Some(404), Down),
        (Some(500), Down),
        (Some(204), Down),
        (Some(0), Down),
        (None, Down),
    ] {
        assert_eq!(health_from_status(status), health, "{status:?}");
    }
}

fn fixture_rows() -> [RowInput<'static>; 4] {
    [
        mac(Some(Down)),
        mac(Some(Down)),
        mac(Some(Down)),
        row("remoteluci", None, None),
    ]
}

#[test]
fn a_foreign_model_blocks_every_grouped_stopped_row_on_its_host() {
    let foreign = [("Local", 8090)];
    assert_eq!(
        row_states(&fixture_rows(), &foreign),
        [
            Blocked {
                by: Foreign { port: 8090 }
            },
            Blocked {
                by: Foreign { port: 8090 }
            },
            Blocked {
                by: Foreign { port: 8090 }
            },
            Stopped,
        ]
    );
}

#[test]
fn the_first_foreign_entry_of_the_host_names_the_port() {
    let foreign = [("remoteluci", 1), ("Local", 8090), ("Local", 8091)];
    assert_eq!(
        row_states(&[mac(Some(Down))], &foreign),
        [Blocked {
            by: Foreign { port: 8090 }
        }]
    );
}

#[test]
fn a_row_blocker_takes_precedence_over_a_foreign_one() {
    let rows = [mac(Some(Down)), mac(Some(Ready))];
    assert_eq!(
        row_states(&rows, &[("Local", 8090)]),
        [Blocked { by: Row(1) }, Running]
    );
}

#[test]
fn a_foreign_model_on_one_host_leaves_another_host_alone() {
    let rows = [row("remoteluci", Some("mac-llm"), Some(Down))];
    assert_eq!(row_states(&rows, &[("Local", 8090)]), [Stopped]);
}

#[test]
fn running_and_starting_rows_and_ungrouped_rows_ignore_a_foreign_model() {
    let rows = [
        mac(Some(Ready)),
        mac(Some(Loading)),
        row(MAC, None, Some(Down)),
    ];
    assert_eq!(
        row_states(&rows, &[("Local", 8090)]),
        [Running, Starting, Stopped]
    );
}
