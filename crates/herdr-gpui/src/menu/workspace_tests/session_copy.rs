use super::*;
use crate::session_copy::tests::{CODEX_ID, snapshot, two_agents};
use crate::sidebar;
use crate::sidebar::layout_tests::SidebarFixture;
use crate::state::ConnectionStatus;
use gpui::{Entity, TestAppContext, VisualTestContext, point, prelude::*, px};
use std::sync::Arc;

fn window_with(
    cx: &mut TestAppContext,
    bindings: Vec<crate::chat::existing::Binding>,
) -> (Entity<HerdrWindow>, &mut VisualTestContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        let view = cx.new(|cx| {
            let mut view = sidebar::layout_tests::fixture_window(window, cx);
            view.live.snapshot = Some(Arc::new(snapshot()));
            view.live.status = ConnectionStatus::Connected;
            view.chats.existing.seed_bindings(bindings);
            view
        });
        cx.observe(&view, |_, _, cx| cx.notify()).detach();
        SidebarFixture(view)
    });
    let view = cx.update(|_, cx| fixture.read(cx).0.clone());
    (view, cx)
}

fn session_actions(view: &HerdrWindow) -> Vec<(WorkspaceMenuAction, String)> {
    view.workspace_items()
        .into_iter()
        .filter(|(action, _)| matches!(action, WorkspaceMenuAction::CopySession(_)))
        .map(|(action, label)| (action, label.to_string()))
        .collect()
}

#[gpui::test]
fn workspace_menu_has_one_labelled_entry_per_agent(cx: &mut TestAppContext) {
    let (view, cx) = window_with(cx, two_agents());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_workspace_menu("w0", point(px(10.), px(10.)), window, cx)
        })
    });
    cx.update(|_, cx| {
        let entries = session_actions(view.read(cx));
        assert_eq!(
            entries,
            [
                (
                    WorkspaceMenuAction::CopySession(0),
                    "claude \u{b7} 184567ac".into()
                ),
                (
                    WorkspaceMenuAction::CopySession(1),
                    "codex \u{b7} 9f3c2b1a".into()
                ),
            ]
        );
    });
}

#[gpui::test]
fn workspace_menu_hides_entries_without_a_session_id(cx: &mut TestAppContext) {
    let (view, cx) = window_with(cx, Vec::new());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_workspace_menu("w0", point(px(10.), px(10.)), window, cx)
        })
    });
    cx.update(|_, cx| assert!(session_actions(view.read(cx)).is_empty()));
}

#[gpui::test]
fn choosing_a_workspace_entry_copies_the_full_session_id(cx: &mut TestAppContext) {
    let (view, cx) = window_with(cx, two_agents());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_workspace_menu("w0", point(px(10.), px(10.)), window, cx);
            view.activate_workspace_menu(WorkspaceMenuAction::CopySession(1), window, cx);
        })
    });
    let copied = cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text()));
    assert_eq!(copied.as_deref(), Some(CODEX_ID));
    cx.update(|_, cx| {
        let view = view.read(cx);
        assert!(view.menu.page.is_none());
        assert!(view.flash.is_some(), "the copied id is flashed");
    });
}
