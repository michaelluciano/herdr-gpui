//! An agent row's name takes line 1 whole; the details sit on line 2 after
//! the agent icon, a `|` and the tab's number.
use super::*;
use crate::config::{Density, LayoutMode};

fn draw(width: f32, setting: &str, cx: &mut gpui::TestAppContext) -> gpui::VisualTestContext {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(Arc::new(snapshot(2)));
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    view.update(cx, |view, cx| {
        view.config.layout.mode = LayoutMode::from(Density::Comfortable);
        view.config.sidebar.size = 12.;
        view.sidebar_width = Some(width);
        view.config.sidebar_layout.agents = toml::from_str(setting).unwrap();
        view.config.usage.inline = false;
        cx.notify();
    });
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    cx.clone()
}

#[gpui::test]
fn the_name_has_the_whole_first_line_and_the_details_the_second(cx: &mut gpui::TestAppContext) {
    let cx = &mut draw(320., "rows = [[\"state_text\"]]", cx);
    let column = cx.debug_bounds("column-agent-p0").unwrap();
    let name = cx.debug_bounds("name-agent-p0").unwrap();
    let icon = cx.debug_bounds("agent-icon-agent-p0").unwrap();
    let sep = cx.debug_bounds("agent-sep-agent-p0").unwrap();
    let tab = cx.debug_bounds("agent-tab-agent-p0").unwrap();
    let status = cx.debug_bounds("status-agent-p0").unwrap();
    assert_eq!(name.left(), column.left());
    assert_eq!(name.right(), column.right());
    assert!(name.bottom() <= icon.top() + px(1.));
    assert!(icon.right() <= sep.left());
    assert!(sep.right() <= tab.left());
    assert!(tab.right() <= status.left());
    assert!(status.top() >= name.bottom());
    assert!(status.right() <= column.right());
    assert_eq!(icon.size, size(px(12.), px(12.)));
}

#[gpui::test]
fn without_a_resolvable_tab_the_tag_and_separator_go(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut snapshot = snapshot(2);
        snapshot.agents[0].tab_id = "gone".into();
        view.live.snapshot = Some(Arc::new(snapshot));
        view.config.sidebar_layout.agents = toml::from_str("rows = [[\"state_text\"]]").unwrap();
        view.config.usage.inline = false;
        view.sidebar_width = Some(320.);
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    view.update(cx, |_, cx| cx.notify());
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("agent-sep-agent-p0").is_none());
    assert!(cx.debug_bounds("agent-tab-agent-p0").is_none());
    assert!(cx.debug_bounds("status-agent-p0").is_some());
}

/// One-line rows keep their status but the name keeps twelve characters
/// first, in every layout.
#[gpui::test]
fn on_one_line_the_name_outlasts_the_status(cx: &mut gpui::TestAppContext) {
    for mode in [LayoutMode::Superset, LayoutMode::Orca, LayoutMode::Minimal] {
        let (view, cx) = cx.add_window_view(|window, cx| {
            let mut view = fixture_window(window, cx);
            let mut snapshot = snapshot(2);
            snapshot.agents[0].name = Some("a-reasonably-long-agent-name".into());
            snapshot.agents[0].display_agent = None;
            view.live.snapshot = Some(Arc::new(snapshot));
            view
        });
        cx.simulate_resize(size(px(800.), px(900.)));
        cx.run_until_parked();
        view.update(cx, |view, cx| {
            view.config.layout.mode = mode;
            view.config.sidebar.size = 12.;
            view.sidebar_width = Some(200.);
            view.config.sidebar_layout.agents =
                toml::from_str("rows = [[\"state_text\"]]").unwrap();
            view.config.usage.inline = false;
            cx.notify();
        });
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
        let name = cx.debug_bounds("name-agent-p0").unwrap();
        assert!(
            f32::from(name.size.width) >= 80.,
            "{mode:?} {:?}",
            name.size
        );
        assert!(cx.debug_bounds("status-agent-p0").is_some(), "{mode:?}");
    }
}
