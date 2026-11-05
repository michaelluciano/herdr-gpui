#![allow(clippy::expect_used)]

//! The Handoffs section lists the newest unclaimed handoff of every project.
use super::handoffs::{handoff, texts, window};

#[gpui::test]
fn one_row_per_project_newest_first_whatever_the_focused_workspace(cx: &mut gpui::TestAppContext) {
    let list = vec![
        handoff("S-kb-new", "kb", "2026-11-03T10:00:00"),
        handoff("S-rem-new", "remora", "2026-11-02T10:00:00"),
        handoff("S-kb-old", "kb", "2026-11-01T10:00:00"),
    ];
    let (_view, cx) = window(cx, None, list);
    let shown = texts(cx);
    assert!(shown.iter().any(|text| text == "S-kb-new"), "{shown:?}");
    assert!(shown.iter().any(|text| text == "S-rem-new"), "{shown:?}");
    assert!(!shown.iter().any(|text| text == "S-kb-old"), "{shown:?}");
    let first = cx.debug_bounds("handoff-row-0").expect("row 0");
    let second = cx.debug_bounds("handoff-row-1").expect("row 1");
    assert!(first.top() < second.top());
    assert!(cx.debug_bounds("handoff-row-2").is_none());
    assert!(shown.iter().any(|text| text == "2"), "the count");
}
