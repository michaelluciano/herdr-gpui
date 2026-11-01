//! Clicking a handoff row copies its file's absolute path and flashes it,
//! instead of opening the file in a code tab.
#![allow(clippy::expect_used)]

use super::handoffs::{handoff, window};
use super::*;
use crate::{
    browser::{Location, Store},
    handoffs::Handoff,
};
use gpui::{Entity, Modifiers, VisualTestContext};

/// Clicks the centre of the element `selector`, then redraws the window.
pub(super) fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let bounds = cx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("no {selector}"));
    cx.simulate_click(bounds.center(), Modifiers::default());
    cx.run_until_parked();
    cx.update(|window, cx| {
        window.refresh();
        full_draw(window, cx).clear(cx);
    });
}

fn clipboard(cx: &mut VisualTestContext) -> Option<String> {
    cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text()))
}

fn flash(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |view, _| {
        view.flash.as_ref().map(|(flash, _)| flash.text.to_string())
    })
}

/// The file of every code tab open in the store.
pub(super) fn code_paths(cx: &mut VisualTestContext) -> Vec<String> {
    cx.update(|_, cx| {
        cx.try_global::<Store>().map_or_else(Vec::new, |store| {
            store
                .tabs()
                .filter_map(|tab| match &tab.location {
                    Some(Location::Code { file }) => Some(file.path.clone()),
                    _ => None,
                })
                .collect()
        })
    })
}

#[gpui::test]
fn clicking_a_row_copies_its_absolute_path_and_flashes_it(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx, None, vec![handoff("S38-next", "kb", "2026-10-31")]);
    let path = view.read_with(cx, |view, _| view.handoffs.board.active[0].path.clone());
    assert!(path.is_absolute(), "{path:?}");
    let wanted = path.to_string_lossy().into_owned();

    click(cx, "handoff-row-0");

    assert_eq!(clipboard(cx).as_deref(), Some(wanted.as_str()));
    assert_eq!(
        flash(&view, cx).as_deref(),
        Some(format!("Copied {wanted}").as_str())
    );
}

#[gpui::test]
fn clicking_a_row_opens_no_code_tab(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = window(cx, None, vec![handoff("S38-next", "kb", "2026-10-31")]);
    click(cx, "handoff-row-0");
    assert!(code_paths(cx).is_empty(), "{:?}", code_paths(cx));
}

#[gpui::test]
fn clicking_a_history_row_copies_its_path_and_opens_no_code_tab(cx: &mut gpui::TestAppContext) {
    let picked = Handoff {
        status: "consumed".into(),
        ..handoff("S37", "kb", "2026-10-31T09:15:00")
    };
    let (view, cx) = window(cx, None, vec![picked]);
    click(cx, "handoffs-history-toggle");
    let wanted = view.read_with(cx, |view, _| {
        view.handoffs.board.history[0]
            .handoff
            .path
            .to_string_lossy()
            .into_owned()
    });

    click(cx, "handoff-history-row-0");

    assert_eq!(clipboard(cx).as_deref(), Some(wanted.as_str()));
    assert!(code_paths(cx).is_empty(), "{:?}", code_paths(cx));
}
