//! The last-edit time sits right of the project name on a row's first line.
#![allow(clippy::expect_used)]

use super::handoffs::{handoff, window};
use crate::handoffs::Handoff;
use std::time::SystemTime;

fn edited_now(id: &str, project: &str) -> Handoff {
    Handoff {
        modified: Some(SystemTime::now()),
        ..handoff(id, project, "2026-11-04T10:00:00")
    }
}

#[gpui::test]
fn the_edit_time_is_right_of_the_project_name_inside_the_row(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = window(cx, None, vec![edited_now("S1", "kb")]);
    let project = cx.debug_bounds("handoff-row-project-0").expect("project");
    let edited = cx.debug_bounds("handoff-row-edited-0").expect("edited");
    let row = cx.debug_bounds("handoff-row-0").expect("row");
    assert!(edited.left() >= project.right(), "{edited:?} {project:?}");
    assert!(edited.right() <= row.right(), "{edited:?} {row:?}");
    assert!(edited.top() < project.bottom() && edited.bottom() > project.top());
}

#[gpui::test]
fn a_history_row_has_one_too_and_a_row_without_an_edit_time_has_none(
    cx: &mut gpui::TestAppContext,
) {
    let picked = Handoff {
        status: "claimed".into(),
        ..edited_now("S2", "remora")
    };
    let bare = handoff("S3", "wb", "2026-11-03T10:00:00");
    let (_view, cx) = window(cx, None, vec![edited_now("S1", "kb"), picked, bare]);
    super::handoff_copy::click(cx, "handoffs-history-toggle");
    assert!(cx.debug_bounds("handoff-history-row-edited-0").is_some());
    assert!(cx.debug_bounds("handoff-row-edited-1").is_none());
}
