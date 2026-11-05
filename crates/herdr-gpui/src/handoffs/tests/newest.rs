//! The active list: the newest handoff of each project, when it is unclaimed.
use super::*;
use crate::handoffs::classify::{Board, Shelf, Shelved, classify};

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
fn each_project_shows_only_its_newest_open_handoff_newest_project_first() {
    // Listed newest first, as the loader returns them.
    let board = classify(vec![
        fixture("kb-new", "kb", "2026-11-03T10:00:00", "open"),
        fixture("rem-new", "remora", "2026-11-02T10:00:00", "open"),
        fixture("kb-old", "kb", "2026-11-01T10:00:00", "open"),
        fixture("rem-old", "remora", "2026-10-30T10:00:00", "open"),
    ]);
    assert_eq!(
        split(&board),
        (
            vec!["kb-new", "rem-new"],
            vec![
                ("kb-old", Shelf::Superseded),
                ("rem-old", Shelf::Superseded)
            ]
        )
    );
}

#[test]
fn a_claimed_newest_leaves_the_project_with_no_active_row() {
    let board = classify(vec![
        fixture("new", "kb", "2026-11-03", "claimed"),
        fixture("old", "kb", "2026-11-02", "open"),
        fixture("older", "kb", "2026-11-01", "open"),
    ]);
    assert_eq!(
        split(&board),
        (
            Vec::new(),
            vec![
                ("new", Shelf::PickedUp),
                ("old", Shelf::Superseded),
                ("older", Shelf::Superseded)
            ]
        )
    );
}

#[test]
fn an_open_newest_with_a_claim_is_picked_up_and_hides_older_ones() {
    let held = Handoff {
        claimed_by: "uuid".into(),
        ..fixture("new", "kb", "2026-11-03", "open")
    };
    let board = classify(vec![held, fixture("old", "kb", "2026-11-02", "open")]);
    assert_eq!(
        split(&board),
        (
            Vec::new(),
            vec![("new", Shelf::PickedUp), ("old", Shelf::Superseded)]
        )
    );
}

#[test]
fn an_older_handoff_without_prev_or_session_is_superseded_by_a_sibling() {
    let board = classify(vec![
        fixture("new", "p", "2026-11-03", "open"),
        fixture("old", "p", "2026-11-02", "open"),
    ]);
    assert_eq!(
        split(&board),
        (vec!["new"], vec![("old", Shelf::Superseded)])
    );
}

#[test]
fn a_handoff_with_no_branch_or_cwd_is_shown() {
    // The kb S46 shape: only the base front matter keys.
    let root = Root::new("bare");
    root.session(
        "kb",
        "S46.md",
        "---\nid: S46\ncreated: 2026-11-04\nstatus: open\n---\nbody\n",
    );
    let board = classify(list_all(root.path()));
    assert_eq!(split(&board), (vec!["S46"], Vec::new()));
}
