//! Dismissing handoffs: the board, the bulk actions, and the saved set.
use super::*;
use crate::handoffs::{
    Handoffs,
    classify::{Board, Shelf, classify_with},
    dismissed,
};
use std::collections::HashSet;

fn shelves(board: &Board) -> Vec<(&str, Shelf)> {
    board
        .history
        .iter()
        .map(|shelved| (shelved.handoff.id.as_str(), shelved.shelf))
        .collect()
}

fn picked(id: &str, project: &str, created: &str) -> Handoff {
    Handoff {
        claimed_by: "who".into(),
        ..fixture(id, project, created, "open")
    }
}

fn mixed() -> Vec<Handoff> {
    vec![
        fixture("kb-new", "kb", "2026-11-03", "open"),
        picked("rem-new", "remora", "2026-11-02"),
        fixture("kb-old", "kb", "2026-11-01", "open"),
        picked("old-pick", "other", "2026-10-30"),
        fixture("rem-old", "remora", "2026-10-29", "open"),
    ]
}

fn paths(list: &[Handoff], ids: &[&str]) -> HashSet<PathBuf> {
    list.iter()
        .filter(|h| ids.contains(&h.id.as_str()))
        .map(|h| h.path.clone())
        .collect()
}

#[test]
fn the_dismissed_label_is_dismissed() {
    assert_eq!(Shelf::Dismissed.label(), "dismissed");
}

#[test]
fn a_dismissed_newest_leaves_its_project_empty_and_older_ones_stay_superseded() {
    let list = mixed();
    let board = classify_with(list.clone(), &paths(&list, &["kb-new"]));
    assert!(board.active.is_empty(), "{:?}", board.active);
    assert_eq!(
        shelves(&board),
        vec![
            ("kb-new", Shelf::Dismissed),
            ("rem-new", Shelf::PickedUp),
            ("kb-old", Shelf::Superseded),
            ("old-pick", Shelf::PickedUp),
            ("rem-old", Shelf::Superseded),
        ]
    );
}

#[test]
fn classify_without_dismissals_is_unchanged() {
    let list = mixed();
    assert_eq!(classify(list.clone()), classify_with(list, &HashSet::new()));
}

#[test]
fn clear_picked_up_dismisses_only_picked_up_rows() {
    let mut state = Handoffs::default();
    state.set_list(mixed());
    assert_eq!(state.board.count(Shelf::PickedUp), 2);
    state.clear_picked_up();
    assert_eq!(
        shelves(&state.board),
        vec![
            ("rem-new", Shelf::Dismissed),
            ("kb-old", Shelf::Superseded),
            ("old-pick", Shelf::Dismissed),
            ("rem-old", Shelf::Superseded),
        ]
    );
    assert_eq!(state.board.active.len(), 1);
    assert_eq!(state.board.count(Shelf::PickedUp), 0);
}

#[test]
fn restore_empties_the_set_and_brings_the_rows_back() {
    let mut state = Handoffs::default();
    state.set_list(mixed());
    let before = state.board.clone();
    state.dismiss(state.board.active[0].path.clone());
    state.clear_picked_up();
    assert_eq!(state.board.count(Shelf::Dismissed), 3);
    state.restore_dismissed();
    assert!(state.dismissed.is_empty());
    assert_eq!(state.board, before);
}

#[test]
fn dismissals_survive_a_new_list() {
    let mut state = Handoffs::default();
    state.set_list(mixed());
    state.dismiss(state.board.active[0].path.clone());
    state.set_list(mixed());
    assert_eq!(state.board.count(Shelf::Dismissed), 1);
}

#[test]
fn the_saved_set_round_trips_through_a_file() {
    let root = Root::new("dismissed");
    let path = root.path().join("nested/handoff-dismissed.json");
    let set: HashSet<PathBuf> = ["/a/b/sessions/x.md", "/c/d/sessions/y.md"]
        .into_iter()
        .map(PathBuf::from)
        .collect();
    crate::state_file::write(&path, &dismissed::to_saved(&set)).unwrap();
    assert_eq!(dismissed::load(&path), set);
    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        serde_json::from_str::<Vec<String>>(&text).unwrap(),
        vec!["/a/b/sessions/x.md", "/c/d/sessions/y.md"]
    );
}

#[test]
fn a_missing_or_damaged_file_loads_empty() {
    let root = Root::new("dismissed-bad");
    assert!(dismissed::load(&root.path().join("none.json")).is_empty());
    let bad = root.file("bad.json", "{not json");
    assert!(dismissed::load(&bad).is_empty());
}
