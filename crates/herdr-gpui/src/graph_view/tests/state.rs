use super::*;
use crate::graph_view::state::GraphState;
use std::sync::atomic::Ordering;

fn history() -> Vec<Commit> {
    vec![commit("b", &["a"]), commit("a", &[])]
}

fn loaded() -> (GraphState, u64) {
    let mut state = GraphState::default();
    let (request, _) = state.begin_request();
    assert!(state.apply(request, history()));
    (state, request)
}

#[test]
fn default_state_is_empty_and_safe() {
    let state = GraphState::default();
    assert_eq!(state.row_count(), 0);
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.head_branch, None);
}

#[test]
fn begin_request_bumps_the_counter_and_cancels_the_old_flag() {
    let mut state = GraphState::default();
    let (first, first_flag) = state.begin_request();
    let (second, second_flag) = state.begin_request();
    assert!(second > first);
    assert!(first_flag.load(Ordering::SeqCst));
    assert!(!second_flag.load(Ordering::SeqCst));
}

#[test]
fn cancel_all_sets_the_current_flag() {
    let mut state = GraphState::default();
    let (_, flag) = state.begin_request();
    state.cancel_all();
    assert!(flag.load(Ordering::SeqCst));
}

#[test]
fn stale_request_is_dropped() {
    let mut state = GraphState::default();
    let (old, _) = state.begin_request();
    let (current, _) = state.begin_request();
    assert!(!state.apply(old, history()));
    assert_eq!(state.row_count(), 0);
    assert!(state.apply(current, history()));
    assert_eq!(state.row_count(), 2);
}

#[test]
fn first_apply_selects_the_first_commit_and_builds_layout() {
    let (state, _) = loaded();
    assert_eq!(state.selected.as_deref(), Some("b"));
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.layout, herdr_vcs::graph::layout(&state.commits));
    assert_eq!(state.layout.rows.len(), state.row_count());
}

#[test]
fn identical_commits_report_no_change_and_keep_layout() {
    let (mut state, _) = loaded();
    let (request, _) = state.begin_request();
    // A sentinel proves the layout is not recomputed for equal commits.
    state.layout.lane_count = 99;
    assert!(!state.apply(request, history()));
    assert_eq!(state.layout.lane_count, 99);
}

#[test]
fn selection_survives_a_refresh_that_adds_a_newer_commit() {
    let (mut state, _) = loaded();
    assert!(state.select("a"));
    let (request, _) = state.begin_request();
    let refreshed = vec![commit("c", &["b"]), commit("b", &["a"]), commit("a", &[])];
    assert!(state.apply(request, refreshed));
    assert_eq!(state.selected.as_deref(), Some("a"));
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn selection_falls_back_to_the_first_commit_when_it_vanishes() {
    let (mut state, _) = loaded();
    assert!(state.select("a"));
    let (request, _) = state.begin_request();
    assert!(state.apply(request, vec![commit("z", &[]), commit("y", &["z"])]));
    assert_eq!(state.selected.as_deref(), Some("z"));
}

#[test]
fn select_reports_change_and_ignores_unknown_ids() {
    let (mut state, _) = loaded();
    assert!(!state.select("b"));
    assert!(!state.select("nope"));
    assert_eq!(state.selected.as_deref(), Some("b"));
    assert!(state.select("a"));
}

#[test]
fn head_branch_comes_from_the_commit_marked_head() {
    let mut state = GraphState::default();
    let (request, _) = state.begin_request();
    let commits = vec![
        commit_with_refs(
            "b",
            &["a"],
            vec![Ref::Head, Ref::LocalBranch("main".to_owned())],
        ),
        commit_with_refs("a", &[], vec![Ref::LocalBranch("old".to_owned())]),
    ];
    assert!(state.apply(request, commits));
    assert_eq!(state.head_branch.as_deref(), Some("main"));
}

#[test]
fn detached_head_has_no_branch() {
    let mut state = GraphState::default();
    let (request, _) = state.begin_request();
    let commits = vec![commit_with_refs(
        "a",
        &[],
        vec![Ref::Head, Ref::Tag("v1".to_owned())],
    )];
    assert!(state.apply(request, commits));
    assert_eq!(state.head_branch, None);
}

#[test]
fn empty_commit_list_clears_selection_without_panicking() {
    let (mut state, _) = loaded();
    let (request, _) = state.begin_request();
    assert!(state.apply(request, Vec::new()));
    assert_eq!(state.row_count(), 0);
    assert_eq!(state.selected, None);
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.head_branch, None);
    assert!(!state.select("a"));
}
