use super::*;
use crate::graph_view::state::GraphState;

fn loaded() -> GraphState {
    let mut state = GraphState::default();
    let (request, _) = state.begin_request();
    let commits = vec![commit("c", &["b"]), commit("b", &["a"]), commit("a", &[])];
    assert!(state.apply(request, commits));
    state
}

#[test]
fn selecting_a_commit_records_its_row_as_the_scroll_target() {
    let mut state = loaded();
    state.take_scroll_target();
    assert!(state.select("a"));
    assert_eq!(state.take_scroll_target(), Some(2));
}

#[test]
fn the_scroll_target_is_cleared_once_taken() {
    let mut state = loaded();
    assert!(state.select("b"));
    assert_eq!(state.take_scroll_target(), Some(1));
    assert_eq!(state.take_scroll_target(), None);
}

#[test]
fn selecting_the_already_selected_commit_sets_no_target() {
    let mut state = loaded();
    state.take_scroll_target();
    assert!(!state.select("c"));
    assert_eq!(state.take_scroll_target(), None);
}

#[test]
fn a_refresh_that_moves_the_selection_records_the_new_row() {
    let mut state = loaded();
    assert!(state.select("b"));
    state.take_scroll_target();
    let (request, _) = state.begin_request();
    // "b" vanished: selection falls back to the first commit.
    assert!(state.apply(request, vec![commit("z", &["y"]), commit("y", &[])]));
    assert_eq!(state.selected.as_deref(), Some("z"));
    assert_eq!(state.take_scroll_target(), Some(0));
}
