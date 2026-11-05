//! The lock at the left of each handoff row pins it to the top of the active list.
#![allow(clippy::expect_used)]

use super::handoff_copy::click;
use super::handoffs::{handoff, window};
use crate::handoffs::Handoff;
use gpui::px;

fn picked(id: &str, project: &str, created: &str) -> Handoff {
    Handoff {
        status: "claimed".into(),
        claimed_by: "who".into(),
        ..handoff(id, project, created)
    }
}

fn list() -> Vec<Handoff> {
    vec![
        handoff("a-new", "alpha", "2026-11-04T10:00:00"),
        handoff("b-new", "beta", "2026-11-03T10:00:00"),
        picked("c-pick", "gamma", "2026-11-02T10:00:00"),
    ]
}

#[gpui::test]
fn every_row_has_an_open_lock_until_it_is_pinned(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx, None, list());
    click(cx, "handoffs-history-toggle");
    for selector in [
        "handoff-row-pin-0",
        "handoff-row-pin-1",
        "handoff-history-row-pin-0",
    ] {
        assert!(cx.debug_bounds(selector).is_some(), "{selector}");
    }
    view.read_with(cx, |view, _| assert!(view.handoffs.pinned.is_empty()));
    let pin = cx.debug_bounds("handoff-row-pin-0").expect("pin");
    let row = cx.debug_bounds("handoff-row-0").expect("row");
    let id = cx.debug_bounds("handoff-row-x-0").expect("x");
    assert!(pin.left() < id.left(), "the lock is left of the x");
    assert!(
        pin.left() - row.left() < row.size.width / 4.,
        "at the left edge"
    );
}

#[gpui::test]
fn clicking_the_lock_pins_without_copying_the_path(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx, None, list());
    click(cx, "handoff-row-pin-1");
    assert_eq!(
        cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text())),
        None,
        "the row's own click must not fire"
    );
    view.read_with(cx, |view, _| {
        let ids: Vec<_> = view.handoffs.board.active.iter().map(|h| &h.id).collect();
        assert_eq!(ids, vec!["b-new", "a-new"]);
        assert_eq!(view.handoffs.pinned.len(), 1);
    });
    // The pinned row is now first, and clicking its lock unpins.
    click(cx, "handoff-row-pin-0");
    view.read_with(cx, |view, _| {
        let ids: Vec<_> = view.handoffs.board.active.iter().map(|h| &h.id).collect();
        assert_eq!(ids, vec!["a-new", "b-new"]);
        assert!(view.handoffs.pinned.is_empty());
    });
}

#[gpui::test]
fn a_pinned_history_row_stays_in_history_and_keeps_its_lock(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx, None, list());
    click(cx, "handoffs-history-toggle");
    click(cx, "handoff-history-row-pin-0");
    view.read_with(cx, |view, _| {
        assert_eq!(view.handoffs.board.active.len(), 2);
        assert_eq!(view.handoffs.board.history[0].handoff.id, "c-pick");
    });
    view.read_with(cx, |view, _| assert_eq!(view.handoffs.pinned.len(), 1));
}

#[gpui::test]
fn the_padlock_svg_has_its_own_ten_pixel_bounds_inside_the_lock_control(
    cx: &mut gpui::TestAppContext,
) {
    let (_view, cx) = window(cx, None, list());
    let check = |cx: &mut gpui::VisualTestContext| {
        let pin = cx.debug_bounds("handoff-row-pin-1").expect("pin");
        let icon = cx.debug_bounds("handoff-row-pin-icon-1").expect("icon");
        assert!(icon.size.width > px(0.) && icon.size.height > px(0.));
        assert_eq!((icon.size.width, icon.size.height), (px(10.), px(10.)));
        assert!(pin.contains(&icon.origin) && pin.contains(&icon.bottom_right()));
    };
    check(cx);
    click(cx, "handoff-row-pin-1");
    check(cx);
}
