use super::*;
use crate::handoffs::{
    classify::{Board, Shelf, Shelved, classify},
    list::list_all,
};

/// The ids of the active handoffs, and `(id, shelf)` of each shelved one.
fn split(board: &Board) -> (Vec<&str>, Vec<(&str, Shelf)>) {
    (
        board.active.iter().map(|h| h.id.as_str()).collect(),
        board
            .history
            .iter()
            .map(|Shelved { shelf, handoff }| (handoff.id.as_str(), *shelf))
            .collect(),
    )
}

#[test]
fn a_lone_open_handoff_is_active() {
    let board = classify(vec![fixture("a", "p", "2026-10-31", "open")]);
    assert_eq!(split(&board), (vec!["a"], Vec::new()));
}

#[test]
fn a_prev_chain_supersedes_every_link_it_names() {
    let a = Handoff {
        prev: "b".into(),
        ..fixture("a", "p", "2026-10-03", "open")
    };
    let b = Handoff {
        prev: "c".into(),
        ..fixture("b", "p", "2026-10-02", "open")
    };
    let c = fixture("c", "p", "2026-10-01", "open");
    let board = classify(vec![a, b, c]);
    assert_eq!(
        split(&board),
        (
            vec!["a"],
            vec![("b", Shelf::Superseded), ("c", Shelf::Superseded)]
        )
    );
}

#[test]
fn a_claimed_handoff_supersedes_the_one_it_picked_up() {
    let picked = Handoff {
        status: "claimed".into(),
        prev: "s36".into(),
        claimed_by: "uuid".into(),
        ..fixture("s37", "kb", "2026-10-02", "claimed")
    };
    let board = classify(vec![picked, fixture("s36", "kb", "2026-10-01", "open")]);
    assert_eq!(
        split(&board),
        (
            Vec::new(),
            vec![("s37", Shelf::PickedUp), ("s36", Shelf::Superseded)]
        )
    );
}

#[test]
fn a_prev_named_in_another_project_does_not_supersede() {
    let kb = fixture("x", "kb", "2026-10-01", "open");
    let remora = Handoff {
        prev: "x".into(),
        ..fixture("y", "remora", "2026-10-02", "open")
    };
    let board = classify(vec![remora, kb]);
    assert_eq!(split(&board), (vec!["y", "x"], Vec::new()));
}

#[test]
fn a_handoff_naming_itself_as_prev_is_not_superseded_by_itself() {
    let board = classify(vec![Handoff {
        prev: "a".into(),
        ..fixture("a", "p", "2026-10-31", "open")
    }]);
    assert_eq!(split(&board), (vec!["a"], Vec::new()));
}

#[test]
fn the_newest_of_one_session_in_one_project_wins() {
    let older = Handoff {
        session_id: "s1".into(),
        ..fixture("x", "wb", "2026-10-01T09:00:00", "open")
    };
    let newer = Handoff {
        session_id: "s1".into(),
        ..fixture("y", "wb", "2026-10-02T09:00:00", "open")
    };
    // Listed newest first, as the loader returns them.
    let board = classify(vec![newer, older]);
    assert_eq!(split(&board), (vec!["y"], vec![("x", Shelf::Superseded)]));
}

#[test]
fn the_same_session_id_in_another_project_is_not_merged() {
    let kb = Handoff {
        session_id: "s1".into(),
        ..fixture("x", "kb", "2026-10-01", "open")
    };
    let remora = Handoff {
        session_id: "s1".into(),
        ..fixture("y", "remora", "2026-10-02", "open")
    };
    let board = classify(vec![remora, kb]);
    assert_eq!(split(&board), (vec!["y", "x"], Vec::new()));
}

#[test]
fn a_created_tie_in_one_session_keeps_the_path_first_one() {
    // Equal `created` falls to the path order `list_all` sorts by.
    let x = Handoff {
        session_id: "s1".into(),
        ..fixture("x", "p", "2026-10-01", "open")
    };
    let y = Handoff {
        session_id: "s1".into(),
        ..fixture("y", "p", "2026-10-01", "open")
    };
    let board = classify(vec![x, y]);
    assert_eq!(split(&board), (vec!["x"], vec![("y", Shelf::Superseded)]));
}

#[test]
fn handoffs_without_a_session_id_never_group() {
    let board = classify(vec![
        fixture("x", "p", "2026-10-02", "open"),
        fixture("y", "p", "2026-10-01", "open"),
    ]);
    assert_eq!(split(&board), (vec!["x", "y"], Vec::new()));
}

#[test]
fn claimed_consumed_or_claimed_by_hides_as_picked_up() {
    let claimed = fixture("a", "p", "2026-10-04", "claimed");
    let consumed = fixture("b", "p", "2026-10-03", "consumed");
    let claimed_by = Handoff {
        claimed_by: "uuid".into(),
        ..fixture("c", "p", "2026-10-02", "open")
    };
    let plain = fixture("d", "p", "2026-10-01", "open");
    let board = classify(vec![claimed, consumed, claimed_by, plain]);
    assert_eq!(
        split(&board),
        (
            vec!["d"],
            vec![
                ("a", Shelf::PickedUp),
                ("b", Shelf::PickedUp),
                ("c", Shelf::PickedUp),
            ]
        )
    );
}

#[test]
fn other_statuses_are_neither_active_nor_shown() {
    let board = classify(
        [
            "accepted", "dormant", "draft", "plan", "proposed", "", "OPEN",
        ]
        .into_iter()
        .enumerate()
        .map(|(n, status)| fixture(&format!("h{n}"), "p", "2026-10-31", status))
        .collect(),
    );
    assert_eq!(split(&board), (Vec::new(), Vec::new()));
}

#[test]
fn real_shaped_headers_classify_as_their_files_say() {
    let root = Root::new("real");
    root.session(
        "claude-setup",
        "2026-10-31-e8da4cd3.md",
        CLAUDE_SETUP_HEADER,
    );
    root.session("kb", "S37-lease-rerun-2026-10-31.md", KB_HEADER_CLAIMED);
    let board = classify(list_all(root.path()));
    assert_eq!(
        split(&board),
        (
            vec!["S-2026-10-31-e8da4cd3"],
            vec![("S37-lease-rerun-2026-10-31", Shelf::PickedUp)]
        )
    );
}

#[test]
fn a_claimed_by_on_another_status_stays_hidden() {
    for status in ["dormant", "accepted", "draft", "plan", "proposed"] {
        let held = Handoff {
            claimed_by: "uuid".into(),
            ..fixture("d", "p", "2026-10-02", status)
        };
        let board = classify(vec![held, fixture("a", "p", "2026-10-01", "open")]);
        assert_eq!(split(&board), (vec!["a"], Vec::new()), "{status}");
    }
}
