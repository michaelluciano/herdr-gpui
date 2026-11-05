//! The per-row remove control and the all view's bulk actions.
#![allow(clippy::expect_used)]

use super::handoff_copy::click;
use super::handoffs::{handoff, texts, window};
use crate::handoffs::Handoff;
use gpui::VisualTestContext;

fn has(shown: &[String], text: &str) -> bool {
    shown.iter().any(|shown| shown == text)
}

fn picked(id: &str, project: &str, created: &str) -> Handoff {
    Handoff {
        status: "claimed".into(),
        claimed_by: "who".into(),
        ..handoff(id, project, created)
    }
}

/// Two active, two picked up, one superseded.
fn list() -> Vec<Handoff> {
    vec![
        handoff("a-new", "alpha", "2026-11-04T10:00:00"),
        handoff("b-new", "beta", "2026-11-03T10:00:00"),
        picked("c-pick", "gamma", "2026-11-02T10:00:00"),
        picked("d-pick", "delta", "2026-11-01T10:00:00"),
        handoff("a-old", "alpha", "2026-10-30T10:00:00"),
    ]
}

fn clipboard(cx: &mut VisualTestContext) -> Option<String> {
    cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text()))
}

#[gpui::test]
fn the_x_dismisses_that_row_without_copying_its_path(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx, None, list());
    click(cx, "handoff-row-x-0");
    assert_eq!(clipboard(cx), None, "the row's own click must not fire");
    view.read_with(cx, |view, _| {
        let ids: Vec<_> = view.handoffs.board.active.iter().map(|h| &h.id).collect();
        assert_eq!(ids, vec!["b-new"]);
        assert_eq!(view.handoffs.dismissed.len(), 1);
    });
    // The project's older handoff does not resurface.
    assert!(cx.debug_bounds("handoff-row-1").is_none());
}

#[gpui::test]
fn the_x_is_on_history_rows_and_dismissed_rows_show_their_label(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = window(cx, None, list());
    click(cx, "handoffs-history-toggle");
    assert!(cx.debug_bounds("handoff-history-row-x-0").is_some());
    click(cx, "handoff-history-row-x-0");
    let shown = texts(cx);
    assert!(
        shown.iter().any(|t| t.ends_with("\u{b7} dismissed")),
        "{shown:?}"
    );
}

#[gpui::test]
fn the_toggle_reads_all_with_the_total_and_the_active_count_stays(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = window(cx, None, list());
    let shown = texts(cx);
    assert!(has(&shown, "all 3"), "{shown:?}");
    assert!(has(&shown, "\u{b7} 2 active"), "{shown:?}");
    assert!(!shown.iter().any(|t| t.starts_with("picked up ")));
    assert!(cx.debug_bounds("handoffs-clear-picked-up").is_none());
}

#[gpui::test]
fn clear_picked_up_and_restore_work_from_the_all_view(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx, None, list());
    click(cx, "handoffs-history-toggle");
    assert!(has(&texts(cx), "Clear picked up (2)"));
    assert!(cx.debug_bounds("handoffs-restore").is_none(), "none yet");

    click(cx, "handoffs-clear-picked-up");
    let shown = texts(cx);
    assert!(cx.debug_bounds("handoffs-clear-picked-up").is_none());
    assert!(has(&shown, "Restore cleared (2)"), "{shown:?}");
    assert!(has(&shown, "all 3"), "total keeps dismissed: {shown:?}");
    view.read_with(cx, |view, _| {
        assert_eq!(view.handoffs.board.active.len(), 2);
    });

    click(cx, "handoffs-restore");
    assert!(cx.debug_bounds("handoffs-restore").is_none());
    assert!(has(&texts(cx), "Clear picked up (2)"));
    view.read_with(cx, |view, _| assert!(view.handoffs.dismissed.is_empty()));
}
