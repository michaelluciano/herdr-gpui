use super::*;
use crate::config::llm_launchers::Kind;

fn llama(label: &str, port: u16) -> Launcher {
    Launcher {
        label: label.into(),
        host: "Local".into(),
        group: Some("mac-llm".into()),
        kind: Kind::Llama {
            port,
            start: "true".into(),
            url: None,
        },
    }
}

#[test]
fn row_look_truth_table() {
    let blocked = RowState::Blocked {
        by: Blocker::Row(0),
    };
    let cases = [
        (RowState::Stopped, false, true, None),
        (RowState::Starting, false, false, Some("starting")),
        (RowState::Running, false, true, Some("running")),
        (blocked, true, false, None),
        (
            RowState::Blocked {
                by: Blocker::Foreign { port: 9 },
            },
            true,
            false,
            None,
        ),
    ];
    for (state, muted, clickable, suffix) in cases {
        assert_eq!(
            row_look(state),
            RowLook {
                muted,
                clickable,
                suffix
            },
            "{state:?}"
        );
    }
}

#[test]
fn blocked_message_names_the_row_or_the_port() {
    let launchers = [llama("coder", 8081), llama("reason", 8082)];
    assert_eq!(
        blocked_message(Blocker::Row(1), &launchers),
        "reason is running. Stop it first."
    );
    assert_eq!(
        blocked_message(Blocker::Foreign { port: 8090 }, &launchers),
        "Another model is running on port 8090. Stop it first."
    );
}

#[test]
fn blocked_message_for_a_missing_row_does_not_panic() {
    assert_eq!(
        blocked_message(Blocker::Row(7), &[llama("coder", 8081)]),
        "Another model is running. Stop it first."
    );
    assert_eq!(
        blocked_message(Blocker::Row(0), &[]),
        "Another model is running. Stop it first."
    );
}

#[test]
fn blocked_rows_take_the_muted_colour_and_others_do_not() {
    let theme = Theme::default();
    assert_ne!(theme.muted, theme.subtext());
    let blocked = row_look(RowState::Blocked {
        by: Blocker::Row(0),
    });
    assert_eq!(label_color(blocked, &theme), theme.muted);
    assert_eq!(
        label_color(row_look(RowState::Stopped), &theme),
        theme.subtext()
    );
    assert_eq!(
        label_color(row_look(RowState::Running), &theme),
        theme.subtext()
    );
}
