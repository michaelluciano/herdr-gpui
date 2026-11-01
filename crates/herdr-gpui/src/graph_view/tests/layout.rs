use super::paint::{bounds, fixture, graph_id, history, open_with};
use super::*;
use crate::graph_view::state::GraphState;
use gpui::{VisualTestContext, px, size};
use herdr_vcs::graph::{Change, ChangeKind};

fn many_lanes(tips: usize) -> Vec<Commit> {
    let mut commits: Vec<Commit> = (0..tips)
        .map(|n| fixture(&format!("t{n:03}"), &[&format!("p{n:03}")], Vec::new()))
        .collect();
    commits.extend((0..tips).map(|n| fixture(&format!("p{n:03}"), &[], Vec::new())));
    commits
}

fn load_files(view: &gpui::Entity<crate::HerdrWindow>, count: usize, cx: &mut VisualTestContext) {
    let id = graph_id(view, cx);
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            let state: &mut GraphState = view.graphs.get_mut(&id).unwrap();
            let commit = state.selected.clone().unwrap();
            let (request, _) = state.begin_files_request();
            let changes = (0..count)
                .map(|n| Change {
                    kind: ChangeKind::Modified,
                    path: format!("src/file{n}.rs"),
                    old_path: None,
                })
                .collect();
            assert!(state.apply_files(request, &commit, changes));
            cx.notify();
        })
    });
    tab::draw(cx);
}

#[gpui::test]
fn a_repo_with_many_lanes_gets_a_capped_gutter(cx: &mut gpui::TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    let (view, cx) = tab::window(cx, Some(folder.path()));
    open_with(&view, many_lanes(30), cx);
    let lanes = view.read_with(cx, |view, _| {
        view.graphs.values().next().unwrap().layout.lane_count
    });
    assert!(lanes >= 30, "fixture should make 30 lanes, got {lanes}");
    let width = f32::from(bounds(cx, "graph-gutter").size.width);
    assert!((width - 120.0).abs() < 0.5, "gutter is {width}px, want 120");
}

#[gpui::test]
fn the_detail_pane_grows_with_the_window(cx: &mut gpui::TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    let (view, cx) = tab::window(cx, Some(folder.path()));
    open_with(&view, history(), cx);
    cx.simulate_resize(size(px(1000.0), px(1400.0)));
    tab::draw(cx);
    let pane = f32::from(bounds(cx, "graph-detail").size.height);
    let tab = f32::from(bounds(cx, "graph-tab").size.height);
    assert!(pane >= 140.0);
    assert!(pane > 300.0, "pane stayed small in a {tab}px tab: {pane}");
    assert!((pane - 0.38 * tab).abs() < 4.0, "pane {pane} vs tab {tab}");
}

#[gpui::test]
fn the_detail_pane_keeps_a_floor_in_a_short_window(cx: &mut gpui::TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    let (view, cx) = tab::window(cx, Some(folder.path()));
    open_with(&view, history(), cx);
    cx.simulate_resize(size(px(1000.0), px(300.0)));
    tab::draw(cx);
    let pane = f32::from(bounds(cx, "graph-detail").size.height);
    assert!(pane >= 140.0, "pane shrank below its floor: {pane}");
}

#[gpui::test]
fn the_changed_files_are_not_clipped_and_scroll(cx: &mut gpui::TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    let (view, cx) = tab::window(cx, Some(folder.path()));
    open_with(&view, history(), cx);
    cx.simulate_resize(size(px(1000.0), px(900.0)));
    load_files(&view, 40, cx);
    let pane = bounds(cx, "graph-detail");
    let first = bounds(cx, "graph-file-0");
    assert!(
        first.origin.y >= pane.origin.y && first.bottom() <= pane.bottom(),
        "first file {first:?} is outside the pane {pane:?}"
    );
    let scroll = bounds(cx, "graph-files-scroll");
    assert!(
        scroll.bottom() <= pane.bottom(),
        "scroll area {scroll:?} spills out of the pane {pane:?}"
    );
    let last = bounds(cx, "graph-file-39");
    assert!(
        last.bottom() > scroll.bottom(),
        "40 files fit without scrolling; the list must scroll inside its area"
    );
}
