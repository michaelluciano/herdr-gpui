//! Pinned handoffs: the order of the active list and the saved set.
use super::*;
use crate::handoffs::{
    Handoffs,
    classify::{Shelf, classify_with},
    pinned,
};
use std::collections::HashSet;

fn open(id: &str, project: &str, created: &str) -> Handoff {
    fixture(id, project, created, "open")
}

fn list() -> Vec<Handoff> {
    vec![
        open("a", "p1", "2026-11-04"),
        open("b", "p2", "2026-11-03"),
        open("c", "p3", "2026-11-02"),
        open("d", "p4", "2026-11-01"),
    ]
}

fn set(list: &[Handoff], ids: &[&str]) -> HashSet<PathBuf> {
    list.iter()
        .filter(|h| ids.contains(&h.id.as_str()))
        .map(|h| h.path.clone())
        .collect()
}

fn active_ids(board: &crate::handoffs::classify::Board) -> Vec<&str> {
    board.active.iter().map(|h| h.id.as_str()).collect()
}

#[test]
fn pinned_handoffs_sort_to_the_top_and_keep_their_relative_order() {
    let list = list();
    let board = classify_with(list.clone(), &HashSet::new()).pinned_first(&set(&list, &["d", "b"]));
    assert_eq!(active_ids(&board), vec!["b", "d", "a", "c"]);
}

#[test]
fn nothing_pinned_leaves_the_order_unchanged() {
    let list = list();
    let board = classify_with(list.clone(), &HashSet::new());
    assert_eq!(board.clone().pinned_first(&HashSet::new()), board);
    assert_eq!(active_ids(&board), vec!["a", "b", "c", "d"]);
}

#[test]
fn a_pin_on_a_non_active_handoff_changes_nothing() {
    let mut list = list();
    list.push(open("a-old", "p1", "2026-10-01"));
    list.push(Handoff {
        claimed_by: "who".into(),
        ..open("z-pick", "p9", "2026-09-01")
    });
    let plain = classify_with(list.clone(), &HashSet::new());
    let pins = set(&list, &["a-old", "z-pick"]);
    let board = classify_with(list, &HashSet::new()).pinned_first(&pins);
    assert_eq!(active_ids(&board), vec!["a", "b", "c", "d"]);
    assert_eq!(board, plain, "history keeps its order too");
    assert!(board.history.iter().any(|s| s.shelf == Shelf::Superseded));
}

#[test]
fn toggling_a_pin_reorders_and_a_second_toggle_restores() {
    let mut state = Handoffs::default();
    state.set_list(list());
    let c = state.board.active[2].path.clone();
    state.toggle_pin(c.clone());
    assert!(state.pinned.contains(&c));
    assert_eq!(active_ids(&state.board), vec!["c", "a", "b", "d"]);
    state.set_list(list());
    assert_eq!(active_ids(&state.board), vec!["c", "a", "b", "d"]);
    state.toggle_pin(c);
    assert!(state.pinned.is_empty());
    assert_eq!(active_ids(&state.board), vec!["a", "b", "c", "d"]);
}

#[test]
fn a_pin_on_a_picked_up_row_returns_when_it_becomes_active_again() {
    let mut state = Handoffs::default();
    let mut shelved = list();
    shelved[3].claimed_by = "who".into();
    state.set_list(shelved);
    let d = state.board.history[0].handoff.path.clone();
    state.toggle_pin(d);
    assert_eq!(active_ids(&state.board), vec!["a", "b", "c"]);
    state.set_list(list());
    assert_eq!(active_ids(&state.board), vec!["d", "a", "b", "c"]);
}

#[test]
fn the_saved_pins_round_trip_through_a_temp_file() {
    let root = Root::new("pinned");
    let path = root.path().join("nested/handoff-pinned.json");
    let pins: HashSet<PathBuf> = ["/c/d/sessions/y.md", "/a/b/sessions/x.md"]
        .into_iter()
        .map(PathBuf::from)
        .collect();
    crate::state_file::write(&path, &pinned::to_saved(&pins)).unwrap();
    assert_eq!(pinned::load(&path), pins);
    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        serde_json::from_str::<Vec<String>>(&text).unwrap(),
        vec!["/a/b/sessions/x.md", "/c/d/sessions/y.md"]
    );
    assert!(pinned::file().is_none_or(|file| file.ends_with("handoff-pinned.json")));
}

#[test]
fn a_missing_or_damaged_pin_file_loads_empty() {
    let root = Root::new("pinned-bad");
    assert!(pinned::load(&root.path().join("none.json")).is_empty());
    assert!(pinned::load(&root.file("bad.json", "{nope")).is_empty());
}
