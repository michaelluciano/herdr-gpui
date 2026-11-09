//! Many listening ports scroll inside their own room instead of crowding the
//! usage readout out of the status bar.
use super::*;
use crate::sidebar::layout_tests::{fixture_window, full_draw};
use crate::usage::{
    Reading,
    model::{Account, Kind, Report, Window},
    registry,
};
use gpui::{TestAppContext, px, size};

fn many_ports_with_usage(
    window: &mut gpui::Window,
    cx: &mut gpui::Context<crate::window::HerdrWindow>,
    count: u16,
) -> crate::window::HerdrWindow {
    let mut view = fixture_window(window, cx);
    view.sidebar_visible = false;
    let snapshot = std::sync::Arc::make_mut(view.live.snapshot.as_mut().unwrap());
    snapshot.focused_workspace_id = Some("w0".into());
    let lines: String = (0..count)
        .map(|i| format!("L 1 *:{} node\n", 3000 + i))
        .collect();
    view.listening_ports
        .seed(Host::Local, parse(&format!("{lines}E 1 w0\n")).unwrap());
    let provider = registry::find("claude").unwrap();
    view.config.usage.show = true;
    view.usage = crate::usage::Usage::showing(
        Host::Local,
        vec![Reading {
            provider,
            report: Some(Report::new(
                provider,
                Account::default(),
                vec![Window::new(Kind::Session, 28., None, None)],
            )),
            error: None,
            access: None,
        }],
    );
    view
}

#[gpui::test]
fn many_ports_never_overlap_the_usage_readout(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|window, cx| many_ports_with_usage(window, cx, 30));
    cx.simulate_resize(size(px(1000.), px(600.)));
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    let bar = cx.debug_bounds("connection-status").unwrap();
    let ports = cx.debug_bounds("listening-ports").unwrap();
    let usage = cx.debug_bounds("usage").unwrap();
    assert!(
        !ports.intersects(&usage),
        "ports {ports:?} overlap usage {usage:?}"
    );
    assert!(
        ports.left() >= bar.left() && ports.right() <= bar.right(),
        "ports {ports:?} leave the bar {bar:?}"
    );
    // The readout and its refresh button keep their own room.
    assert!(
        ports.left() - usage.left() >= px(96.),
        "usage squeezed to {usage:?} beside {ports:?}"
    );
}

#[gpui::test]
fn many_ports_scroll_sideways_to_reach_the_last_one(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|window, cx| many_ports_with_usage(window, cx, 30));
    cx.simulate_resize(size(px(1000.), px(600.)));
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    let scroll = cx.debug_bounds("listening-ports-scroll").unwrap();
    let first = cx.debug_bounds("port-local-w0-3000").unwrap();
    let last = cx.debug_bounds("port-local-w0-3029").unwrap();
    // Content runs past the scroll box, so only scrolling reaches the last chip.
    assert!(last.right() > scroll.right(), "{last:?} fits in {scroll:?}");
    assert!(first.left() >= scroll.left());
}
