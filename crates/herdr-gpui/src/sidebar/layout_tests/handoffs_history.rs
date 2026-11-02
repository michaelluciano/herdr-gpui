//! The Handoffs header controls: folding the body, the picked-up and
//! superseded history, and when the section shows at all.
#![allow(clippy::expect_used)]

use super::handoffs::{handoff, texts, window};
use super::*;
use crate::handoffs::Handoff;
use gpui::{Modifiers, VisualTestContext};

/// Two active handoffs, then one picked up and one superseded behind them.
fn mixed() -> Vec<Handoff> {
    let setup = handoff(
        "S-2026-10-31-e8da4cd3",
        "claude-setup",
        "2026-10-31T13:41:42",
    );
    let newer = Handoff {
        prev: "S-old".into(),
        ..handoff("S-new", "remora", "2026-10-30T11:00:00")
    };
    let picked = Handoff {
        status: "claimed".into(),
        claimed_by: "184567ac".into(),
        ..handoff("S37", "kb", "2026-10-31T09:15:00")
    };
    let old = handoff("S-old", "remora", "2026-10-29T10:00:00");
    vec![setup, newer, picked, old]
}

/// Redraws the window and forgets the text drawn before.
fn redraw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        cx.default_global::<TextProbes>().0.clear();
        window.refresh();
        full_draw(window, cx).clear(cx);
    });
}

/// Clicks the centre of the element `selector`, then redraws.
fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let bounds = cx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("no {selector}"));
    cx.simulate_click(bounds.center(), Modifiers::default());
    cx.run_until_parked();
    redraw(cx);
}

fn has(shown: &[String], text: &str) -> bool {
    shown.iter().any(|shown| shown == text)
}

#[gpui::test]
fn the_title_folds_and_unfolds_the_body_and_keeps_the_header(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx, None, mixed());
    assert!(has(&texts(cx), "\u{25be}"), "expanded chevron");
    assert!(cx.debug_bounds("handoff-row-0").is_some());

    click(cx, "handoffs-toggle");
    assert!(view.read_with(cx, |view, _| view.handoffs.collapsed));
    assert!(
        cx.debug_bounds("handoffs-section").is_some(),
        "header stays"
    );
    assert!(cx.debug_bounds("handoffs-toggle").is_some());
    assert!(cx.debug_bounds("handoffs-scroll").is_none());
    assert!(cx.debug_bounds("handoff-row-0").is_none());
    assert!(has(&texts(cx), "\u{25b8}"), "folded chevron");

    click(cx, "handoffs-toggle");
    assert!(!view.read_with(cx, |view, _| view.handoffs.collapsed));
    assert!(cx.debug_bounds("handoff-row-0").is_some());
    assert!(has(&texts(cx), "\u{25be}"), "expanded chevron again");
}

#[gpui::test]
fn the_history_control_lists_shelved_rows_below_the_active_ones(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx, None, mixed());
    assert!(cx.debug_bounds("handoff-history-row-0").is_none(), "folded");

    click(cx, "handoffs-history-toggle");
    assert!(view.read_with(cx, |view, _| view.handoffs.history_open));
    let last_active = cx.debug_bounds("handoff-row-0").expect("no active row");
    let first_shelved = cx
        .debug_bounds("handoff-history-row-0")
        .expect("no history row");
    assert!(last_active.bottom() <= first_shelved.top(), "below");
    assert!(cx.debug_bounds("handoff-history-row-1").is_some());
    assert!(cx.debug_bounds("handoff-history-row-2").is_none());

    let shown = texts(cx);
    assert!(
        has(&shown, "kb \u{b7} 10-31 09:15 \u{b7} picked up"),
        "{shown:?}"
    );
    assert!(
        has(&shown, "remora \u{b7} 10-29 10:00 \u{b7} superseded"),
        "{shown:?}"
    );
    assert!(has(&shown, "claude-setup \u{b7} 10-31 13:41"), "{shown:?}");
}

#[gpui::test]
fn the_history_and_the_title_toggle_separately(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx, None, mixed());
    let state = |cx: &mut VisualTestContext| {
        view.read_with(cx, |view, _| {
            (view.handoffs.collapsed, view.handoffs.history_open)
        })
    };
    click(cx, "handoffs-history-toggle");
    assert_eq!(state(cx), (false, true), "history opens, body stays");

    click(cx, "handoffs-toggle");
    assert_eq!(state(cx), (true, true), "title folds, history stays set");
    assert!(
        cx.debug_bounds("handoff-history-row-0").is_none(),
        "a folded body hides its history too"
    );

    click(cx, "handoffs-history-toggle");
    assert_eq!(
        state(cx),
        (true, false),
        "history closes, body stays folded"
    );
}

#[gpui::test]
fn history_alone_keeps_the_header_with_no_active_rows(cx: &mut gpui::TestAppContext) {
    let picked = Handoff {
        status: "consumed".into(),
        ..handoff("S37", "kb", "2026-10-31T09:15:00")
    };
    let (_view, cx) = window(cx, None, vec![picked]);
    assert!(cx.debug_bounds("handoffs-section").is_some());
    assert!(cx.debug_bounds("handoffs-toggle").is_some());
    assert!(cx.debug_bounds("handoffs-history-toggle").is_some());
    assert!(cx.debug_bounds("handoff-row-0").is_none(), "no active rows");
    assert!(cx.debug_bounds("handoff-history-row-0").is_none(), "folded");

    click(cx, "handoffs-history-toggle");
    assert!(cx.debug_bounds("handoff-history-row-0").is_some());
    assert!(cx.debug_bounds("handoff-row-0").is_none());
}

#[gpui::test]
fn nothing_active_or_shelved_hides_the_section(cx: &mut gpui::TestAppContext) {
    let accepted = Handoff {
        status: "accepted".into(),
        ..handoff("S9", "kb", "2026-10-31T09:00:00")
    };
    let (_view, cx) = window(cx, None, vec![accepted]);
    assert!(cx.debug_bounds("handoffs-section").is_none());
    assert!(cx.debug_bounds("handoffs-toggle").is_none());
    assert!(cx.debug_bounds("spaces-section").is_some());
}
