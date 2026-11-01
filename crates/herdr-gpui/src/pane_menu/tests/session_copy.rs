use super::*;
use crate::session_copy::tests::{CLAUDE_ID, binding, snapshot};
use crate::sidebar::layout_tests::SidebarFixture;
use gpui::{Entity, TestAppContext, VisualTestContext, point, px};
use herdr_chat_protocol::Agent;

fn window_with(cx: &mut TestAppContext) -> (Entity<HerdrWindow>, &mut VisualTestContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        let view = cx.new(|cx| {
            let mut view = crate::sidebar::layout_tests::fixture_window(window, cx);
            view.live.snapshot = Some(Arc::new(snapshot()));
            view.live.status = crate::state::ConnectionStatus::Connected;
            view.chats
                .existing
                .seed_bindings(vec![binding("p0", Agent::Claude, CLAUDE_ID)]);
            view
        });
        cx.observe(&view, |_, _, cx| cx.notify()).detach();
        SidebarFixture(view)
    });
    let view = cx.update(|_, cx| fixture.read(cx).0.clone());
    (view, cx)
}

#[gpui::test]
fn pane_menu_offers_the_session_only_when_the_pane_has_one(cx: &mut TestAppContext) {
    let (view, cx) = window_with(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_pane_menu("p0", point(px(10.), px(10.)), window, cx)
        })
    });
    cx.update(|_, cx| {
        let view = view.read(cx);
        let pane = view.menu.pane.as_ref().unwrap();
        assert!(
            pane.actions()
                .iter()
                .any(|a| matches!(a, Action::CopySession))
        );
        assert_eq!(
            Action::CopySession.label(&pane.target),
            "claude \u{b7} 184567ac"
        );
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.dismiss_menu(window, cx);
            view.open_pane_menu("p1", point(px(10.), px(10.)), window, cx)
        })
    });
    cx.update(|_, cx| {
        let view = view.read(cx);
        let pane = view.menu.pane.as_ref().unwrap();
        assert!(
            !pane
                .actions()
                .iter()
                .any(|a| matches!(a, Action::CopySession))
        );
    });
}

#[gpui::test]
fn choosing_the_pane_entry_copies_the_full_session_id(cx: &mut TestAppContext) {
    let (view, cx) = window_with(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_pane_menu("p0", point(px(10.), px(10.)), window, cx);
            view.activate_pane_menu(Action::CopySession, window, cx);
        })
    });
    let copied = cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text()));
    assert_eq!(copied.as_deref(), Some(CLAUDE_ID));
}
