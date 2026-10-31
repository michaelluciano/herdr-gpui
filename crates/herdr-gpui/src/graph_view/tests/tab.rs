use crate::{
    HerdrWindow,
    browser::{Location, Store},
    controls::Command,
    sidebar::layout_tests::{fixture_window, full_draw, snapshot},
    state::ConnectionStatus,
};
use gpui::{Entity, VisualTestContext};
use std::sync::Arc;

pub(super) fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
}

/// Workspace `w0` focused on the owned local daemon, in `directory` on
/// `main`, with the Git chip following it.
pub(super) fn window<'a>(
    cx: &'a mut gpui::TestAppContext,
    directory: Option<&std::path::Path>,
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext) {
    let directory = directory.map(std::path::Path::to_owned);
    cx.add_window_view(move |window, cx| {
        let mut view = fixture_window(window, cx);
        let mut shown = snapshot(2);
        shown.focused_workspace_id = Some("w0".into());
        shown.focused_tab_id = Some("t0".into());
        // The daemon reports no branch for a folder Git does not know.
        for space in &mut shown.workspaces {
            space.branch = None;
            space.worktree = None;
        }
        if let Some(directory) = &directory
            && let Some(space) = shown
                .workspaces
                .iter_mut()
                .find(|space| space.workspace_id == "w0")
        {
            space.new_workspace_cwd = directory.to_str().unwrap_or_default().into();
            space.branch = Some("main".into());
        }
        view.live.snapshot = Some(Arc::new(shown));
        view.live.status = ConnectionStatus::Connected;
        view.live.local_daemon_peer = true;
        view.update_git();
        view
    })
}

pub(super) fn open(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.command(Command::OpenGraph, window, cx))
    });
    draw(cx);
}

fn graph_tabs(cx: &mut VisualTestContext) -> usize {
    cx.update(|_, cx| {
        cx.try_global::<Store>().map_or(0, |store| {
            store
                .tabs()
                .filter(|tab| matches!(tab.location, Some(Location::Graph { .. })))
                .count()
        })
    })
}

#[gpui::test]
fn opening_the_graph_location_draws_the_graph_and_not_a_review(cx: &mut gpui::TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    let (view, cx) = window(cx, Some(folder.path()));
    draw(cx);
    assert!(cx.debug_bounds("graph-tab").is_none());

    open(&view, cx);
    assert_eq!(graph_tabs(cx), 1);
    assert!(cx.debug_bounds("graph-tab").is_some());
    assert!(view.read_with(cx, |view, _| view.reviews.is_empty()));
    assert!(
        cx.debug_bounds("review-tab").is_none(),
        "a Graph tab must not fall through to the review render"
    );

    // Opening it again brings back the same tab.
    open(&view, cx);
    assert_eq!(graph_tabs(cx), 1);
}

#[gpui::test]
fn closing_the_graph_tab_drops_its_state(cx: &mut gpui::TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    let (view, cx) = window(cx, Some(folder.path()));
    open(&view, cx);
    let id = view.read_with(cx, |view, _| *view.graphs.keys().next().unwrap());
    cx.update(|_, cx| Store::update(cx, |store| store.close(id)));
    cx.update(|_, cx| view.update(cx, |view, cx| view.poll_graphs(cx)));
    assert!(view.read_with(cx, |view, _| view.graphs.is_empty()));
}

#[gpui::test]
fn without_a_local_checkout_the_graph_says_so_and_opens_nothing(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx, None);
    open(&view, cx);
    assert_eq!(graph_tabs(cx), 0);
    let flash = cx.update(|_, cx| {
        view.read(cx)
            .flash
            .as_ref()
            .map(|(flash, _)| flash.text.clone())
    });
    assert_eq!(flash.as_deref(), Some("No local checkout to show"));
}
