use super::*;
use crate::graph_view::{load::GRAPH_REFRESH, state::GraphState};
use herdr_vcs::graph::{Change, ChangeKind};
use std::{sync::atomic::Ordering, time::Duration, time::Instant};

fn change(path: &str) -> Change {
    Change {
        kind: ChangeKind::Added,
        path: path.to_owned(),
        old_path: None,
    }
}

fn loaded() -> GraphState {
    let mut state = GraphState::default();
    let (request, _) = state.begin_request();
    assert!(state.apply(request, vec![commit("b", &["a"]), commit("a", &[])]));
    state
}

#[test]
fn refresh_is_due_when_never_loaded() {
    let state = GraphState::default();
    assert!(state.refresh_due(Instant::now(), GRAPH_REFRESH));
}

#[test]
fn refresh_is_due_only_once_the_interval_passed() {
    let mut state = GraphState::default();
    let start = Instant::now();
    state.mark_loaded(start);
    assert!(!state.refresh_due(start + Duration::from_secs(1), GRAPH_REFRESH));
    assert!(state.refresh_due(start + GRAPH_REFRESH, GRAPH_REFRESH));
    assert!(state.refresh_due(start + GRAPH_REFRESH * 2, GRAPH_REFRESH));
}

#[test]
fn begin_request_marks_the_load_time() {
    let mut state = GraphState::default();
    assert_eq!(state.last_load, None);
    state.begin_request();
    assert!(state.last_load.is_some());
}

#[test]
fn apply_files_stores_the_list_for_the_selected_commit() {
    let mut state = loaded();
    let (request, _) = state.begin_files_request();
    assert!(state.apply_files(request, "b", vec![change("x")]));
    assert_eq!(state.files.as_ref().map(|f| f.commit.as_str()), Some("b"));
    // The same list again changes nothing.
    let (request, _) = state.begin_files_request();
    assert!(!state.apply_files(request, "b", vec![change("x")]));
}

#[test]
fn apply_files_drops_a_stale_request() {
    let mut state = loaded();
    let (old, _) = state.begin_files_request();
    let (new, _) = state.begin_files_request();
    assert!(!state.apply_files(old, "b", vec![change("old")]));
    assert!(state.files.is_none());
    assert!(state.apply_files(new, "b", vec![change("new")]));
}

#[test]
fn apply_files_drops_a_commit_that_is_not_selected() {
    let mut state = loaded();
    let (request, _) = state.begin_files_request();
    assert!(!state.apply_files(request, "a", vec![change("x")]));
    assert!(state.files.is_none());
    let (request, _) = state.begin_files_request();
    assert!(state.apply_files(request, "b", vec![change("x")]));
}

#[test]
fn selecting_another_commit_clears_the_files() {
    let mut state = loaded();
    let (request, _) = state.begin_files_request();
    assert!(state.apply_files(request, "b", vec![change("x")]));
    assert!(state.select("a"));
    assert!(state.files.is_none());
}

#[test]
fn begin_files_request_cancels_the_previous_flag() {
    let mut state = loaded();
    let (first, first_flag) = state.begin_files_request();
    let (second, second_flag) = state.begin_files_request();
    assert!(second > first);
    assert!(first_flag.load(Ordering::SeqCst));
    assert!(!second_flag.load(Ordering::SeqCst));
}
