//! A Claude agent's row shows its model letter beside the icon and its context
//! size at the row's far right, in every row layout; no stats, no marks.
use super::*;
use crate::config::{Density, LayoutMode};

fn drawn(
    mode: LayoutMode,
    stats: bool,
    context: u64,
    cx: &mut gpui::TestAppContext,
) -> gpui::VisualTestContext {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(Arc::new(snapshot(2)));
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    view.update(cx, |view, cx| {
        view.config.layout.mode = mode;
        view.config.sidebar.size = 12.;
        view.sidebar_width = Some(320.);
        view.config.usage.inline = false;
        if stats {
            view.agent_activity
                .set_test_stats("p0", "claude-opus-5", context);
            view.agent_activity.set_test_limit(250_000);
        }
        cx.notify();
    });
    cx.update(|window, cx| {
        cx.default_global::<TextProbes>().0.clear();
        full_draw(window, cx).clear(cx);
    });
    cx.clone()
}

fn texts(cx: &mut gpui::VisualTestContext) -> [bool; 2] {
    cx.update(|_, cx| {
        let probes = &cx.global::<TextProbes>().0;
        ["O", "115k"].map(|text| probes.contains_key(text))
    })
}

#[gpui::test]
fn the_default_row_shows_the_letter_beside_the_icon_and_the_context_at_its_edge(
    cx: &mut gpui::TestAppContext,
) {
    let cx = &mut drawn(LayoutMode::from(Density::Comfortable), true, 117_300, cx);
    assert_eq!(texts(cx), [true, true]);
    let row = cx.debug_bounds("row-agent-p0").unwrap();
    let icon = cx.debug_bounds("agent-icon-agent-p0").unwrap();
    let letter = cx.debug_bounds("agent-model-letter").unwrap();
    let context = cx.debug_bounds("agent-context").unwrap();
    let column = cx.debug_bounds("column-agent-p0").unwrap();
    assert!(letter.left() >= icon.right(), "{letter:?} {icon:?}");
    assert_eq!(letter.center().y, icon.center().y);
    assert!(context.left() >= column.right(), "{context:?} {column:?}");
    let inset = row.right() - context.right();
    assert!(inset > px(0.) && inset <= px(24.), "{inset:?}");
    // Text starts after the letter, so the two never overlap.
    let name = cx.debug_bounds("name-agent-p0").unwrap();
    let tab = cx.debug_bounds("agent-tab-agent-p0").unwrap();
    assert!(
        letter.right() <= name.left() || letter.right() <= tab.left(),
        "{letter:?} {name:?}"
    );
}

#[gpui::test]
fn without_stats_neither_mark_exists(cx: &mut gpui::TestAppContext) {
    let cx = &mut drawn(LayoutMode::from(Density::Comfortable), false, 0, cx);
    assert!(cx.debug_bounds("agent-model-letter").is_none());
    assert!(cx.debug_bounds("agent-context").is_none());
    assert_eq!(texts(cx), [false, false]);
}

#[gpui::test]
fn every_layout_shows_both_marks_and_keeps_the_context_in_the_row(cx: &mut gpui::TestAppContext) {
    for mode in [
        LayoutMode::from(Density::Compact),
        LayoutMode::Superset,
        LayoutMode::Orca,
        LayoutMode::Minimal,
    ] {
        let cx = &mut drawn(mode, true, 117_300, cx);
        assert_eq!(texts(cx), [true, true], "{mode:?}");
        let letter = cx.debug_bounds("agent-model-letter").unwrap();
        let context = cx.debug_bounds("agent-context").unwrap();
        assert!(letter.right() <= context.left(), "{mode:?}");
        assert!(context.right() <= px(320.), "{mode:?} {context:?}");
        assert!(context.size.width > px(0.), "{mode:?}");
    }
}
