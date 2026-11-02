//! The Handoffs section lists only the focused workspace's current handoff.
use super::handoffs::{handoff, texts, window};
use crate::handoffs::Handoff;

#[gpui::test]
fn a_newer_handoff_on_another_branch_stays_hidden(cx: &mut gpui::TestAppContext) {
    let other = Handoff {
        branch: "fix/elsewhere".into(),
        ..handoff("S-other", "remora", "2026-11-01T10:00:00")
    };
    let mine = handoff("S-mine", "remora", "2026-11-01T09:00:00");
    let (_view, cx) = window(cx, None, vec![other, mine]);
    let shown = texts(cx);
    assert!(shown.iter().any(|text| text == "S-mine"), "{shown:?}");
    assert!(!shown.iter().any(|text| text == "S-other"), "{shown:?}");
    assert!(cx.debug_bounds("handoff-row-1").is_none());
}

#[gpui::test]
fn no_handoff_on_the_branch_and_no_history_draws_no_section(cx: &mut gpui::TestAppContext) {
    let other = Handoff {
        branch: "fix/elsewhere".into(),
        ..handoff("S-other", "remora", "2026-11-01T10:00:00")
    };
    let (_view, cx) = window(cx, None, vec![other]);
    assert!(cx.debug_bounds("handoffs-section").is_none());
}
