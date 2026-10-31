use super::tab::{draw, open, window};
use super::*;
use crate::{HerdrWindow, browser::TabId};
use gpui::{Bounds, Entity, Modifiers, Pixels, VisualTestContext};

pub(super) fn fixture(id: &str, parents: &[&str], refs: Vec<Ref>) -> Commit {
    Commit {
        subject: format!("subject {id}"),
        author: "Ada".to_owned(),
        time: 1_700_000_000,
        ..commit_with_refs(id, parents, refs)
    }
}

/// Newest first: a merge of two lines, then a linear run of three.
pub(super) fn history() -> Vec<Commit> {
    vec![
        fixture(
            "m000",
            &["c300", "s100"],
            vec![Ref::Head, Ref::LocalBranch("main".into())],
        ),
        fixture("c300", &["c200"], Vec::new()),
        fixture("s100", &["c100"], Vec::new()),
        fixture("c200", &["c100"], Vec::new()),
        fixture("c100", &[], Vec::new()),
    ]
}

pub(super) fn graph_id(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> TabId {
    view.read_with(cx, |view, _| *view.graphs.keys().next().unwrap())
}

/// Open a graph tab and load `commits` into it as a finished request.
pub(super) fn open_with(
    view: &Entity<HerdrWindow>,
    commits: Vec<Commit>,
    cx: &mut VisualTestContext,
) {
    open(view, cx);
    let id = graph_id(view, cx);
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            let state = view.graphs.get_mut(&id).unwrap();
            let (request, _) = state.begin_request();
            state.apply(request, commits);
            cx.notify();
        })
    });
    draw(cx);
}

pub(super) fn bounds(cx: &mut VisualTestContext, name: &'static str) -> Bounds<Pixels> {
    let found = cx.debug_bounds(name);
    assert!(found.is_some(), "no element drew the selector {name:?}");
    found.unwrap()
}

#[gpui::test]
fn graph_tab_draws_one_row_per_commit_in_view(cx: &mut gpui::TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    let (view, cx) = window(cx, Some(folder.path()));
    open_with(&view, history(), cx);
    let rows: Vec<_> = ["graph-row-0", "graph-row-1", "graph-row-2"]
        .into_iter()
        .map(|name| bounds(cx, name))
        .collect();
    for pair in rows.windows(2) {
        assert!(
            pair[0].origin.y < pair[1].origin.y,
            "rows stack top to bottom"
        );
        assert_eq!(pair[0].size.height, pair[1].size.height);
        assert_eq!(pair[0].bottom(), pair[1].origin.y, "rows touch");
    }
}

#[gpui::test]
fn graph_rows_have_a_gutter_as_wide_as_the_layout(cx: &mut gpui::TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    let (view, cx) = window(cx, Some(folder.path()));
    open_with(&view, history(), cx);
    let lanes = view.read_with(cx, |view, _| {
        view.graphs.values().next().unwrap().layout.lane_count
    });
    assert!(lanes >= 2, "the merge fixture needs two lanes");
    let width = f32::from(bounds(cx, "graph-gutter").size.width);
    assert!(width > 0.0);
    let lane_width = width / lanes as f32;
    assert!(
        (lane_width - lane_width.round()).abs() < 0.01 && lane_width >= 1.0,
        "gutter width {width} is not a whole multiple of {lanes} lanes"
    );
}

#[gpui::test]
fn clicking_a_row_selects_it(cx: &mut gpui::TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    let (view, cx) = window(cx, Some(folder.path()));
    open_with(&view, history(), cx);
    let id = graph_id(&view, cx);
    let before = view.read_with(cx, |view, _| view.graphs[&id].selected.clone());
    assert_eq!(before.as_deref(), Some("m000"));
    let row = bounds(cx, "graph-row-1");
    cx.simulate_click(row.center(), Modifiers::default());
    let after = view.read_with(cx, |view, _| view.graphs[&id].selected.clone());
    assert_eq!(after.as_deref(), Some("c300"));
}

#[gpui::test]
fn detail_pane_shows_the_selected_commit(cx: &mut gpui::TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    let (view, cx) = window(cx, Some(folder.path()));
    open_with(&view, history(), cx);
    assert!(cx.debug_bounds("graph-detail").is_some(), "no detail pane");
    assert!(cx.debug_bounds("graph-detail-m000").is_some());
    assert!(cx.debug_bounds("graph-detail-c300").is_none());

    let id = graph_id(&view, cx);
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.graphs.get_mut(&id).unwrap().select("c300");
            cx.notify();
        })
    });
    draw(cx);
    assert!(cx.debug_bounds("graph-detail-c300").is_some());
    assert!(cx.debug_bounds("graph-detail-m000").is_none());
}

#[gpui::test]
fn an_empty_graph_draws_an_empty_state_not_rows(cx: &mut gpui::TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    let (view, cx) = window(cx, Some(folder.path()));
    open_with(&view, Vec::new(), cx);
    assert!(cx.debug_bounds("graph-empty").is_some(), "no empty state");
    assert!(cx.debug_bounds("graph-row-0").is_none());
}

#[gpui::test]
fn graph_tab_takes_focus_when_shown(cx: &mut gpui::TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    let (view, cx) = window(cx, Some(folder.path()));
    open_with(&view, history(), cx);
    assert!(
        cx.debug_bounds("graph-tab-focused").is_some(),
        "the graph tab does not hold the keyboard"
    );
}
