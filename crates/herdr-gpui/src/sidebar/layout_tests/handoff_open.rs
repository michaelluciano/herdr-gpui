//! Right-clicking a handoff row opens its file in a code tab. Left-click only
//! copies the path, and a right-click must not also open a pane or workspace menu.
#![allow(clippy::expect_used)]

use super::handoff_copy::{click, code_paths};
use super::handoffs::{handoff, window};
use super::*;
use crate::handoffs::Handoff;
use gpui::{ClipboardItem, Entity, Modifiers, MouseButton, VisualTestContext};

/// Right-clicks the centre of the element `selector`, then redraws the window.
fn right_click(cx: &mut VisualTestContext, selector: &'static str) {
    let position = cx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("no {selector}"))
        .center();
    cx.simulate_mouse_down(position, MouseButton::Right, Modifiers::default());
    cx.simulate_mouse_up(position, MouseButton::Right, Modifiers::default());
    cx.run_until_parked();
    cx.update(|window, cx| {
        window.refresh();
        full_draw(window, cx).clear(cx);
    });
}

fn clipboard(cx: &mut VisualTestContext) -> Option<String> {
    cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text()))
}

fn path_of(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    active: bool,
    index: usize,
) -> String {
    view.read_with(cx, |view, _| {
        let handoff = if active {
            &view.handoffs.board.active[index]
        } else {
            &view.handoffs.board.history[index].handoff
        };
        handoff.path.to_string_lossy().into_owned()
    })
}

#[gpui::test]
fn right_clicking_a_row_opens_its_file_in_a_code_tab(cx: &mut gpui::TestAppContext) {
    let list = vec![
        handoff("S38-next", "kb", "2026-10-31"),
        handoff("S39-later", "kb", "2026-10-30"),
    ];
    let (view, cx) = window(cx, None, list);
    let wanted = path_of(&view, cx, true, 1);

    right_click(cx, "handoff-row-1");

    assert_eq!(code_paths(cx), [wanted]);
}

#[gpui::test]
fn right_clicking_a_row_leaves_the_clipboard_alone(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = window(cx, None, vec![handoff("S38-next", "kb", "2026-10-31")]);
    cx.update(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string("sentinel".into())));

    right_click(cx, "handoff-row-0");

    assert_eq!(clipboard(cx).as_deref(), Some("sentinel"));
}

#[gpui::test]
fn right_clicking_a_row_opens_no_parent_menu(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx, None, vec![handoff("S38-next", "kb", "2026-10-31")]);

    right_click(cx, "handoff-row-0");

    assert_eq!(view.read_with(cx, |view, _| view.menu.page), None);
}

#[gpui::test]
fn right_clicking_a_history_row_opens_its_file_in_a_code_tab(cx: &mut gpui::TestAppContext) {
    let picked = Handoff {
        status: "consumed".into(),
        ..handoff("S37", "kb", "2026-10-31T09:15:00")
    };
    let (view, cx) = window(cx, None, vec![picked]);
    click(cx, "handoffs-history-toggle");
    let wanted = path_of(&view, cx, false, 0);

    right_click(cx, "handoff-history-row-0");

    assert_eq!(code_paths(cx), [wanted]);
    assert_eq!(view.read_with(cx, |view, _| view.menu.page), None);
}
