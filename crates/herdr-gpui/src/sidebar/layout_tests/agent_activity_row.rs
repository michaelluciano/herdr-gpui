use super::*;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// A Claude agent's row shows what its hooks report: in the default rows a
/// status mark and the time on the details line, in the one-line rows the
/// words; an agent without a report shows the daemon's status.
#[gpui::test]
fn a_claude_row_shows_its_activity_label(cx: &mut gpui::TestAppContext) {
    let dir = std::env::temp_dir().join(format!("herdr-activity-row-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    std::fs::write(
        dir.join("p0.json"),
        format!(
            r#"{{"state":"thinking","since":{},"subagents":0,"tool":""}}"#,
            now - 130
        ),
    )
    .unwrap();
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(Arc::new(snapshot(2)));
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    let draw = |cx: &mut gpui::VisualTestContext| {
        cx.update(|window, cx| {
            cx.default_global::<TextProbes>().0.clear();
            full_draw(window, cx).clear(cx);
            let probes = &cx.global::<TextProbes>().0;
            ["2m", "thinking · 2m", "working"].map(|text| probes.contains_key(text))
        })
    };
    // Before the first read the default row shows no status word.
    assert_eq!(draw(cx), [false, false, false]);
    let path = dir.clone();
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.poll_agent_activity_at(Instant::now(), path, cx)
        })
    });
    cx.run_until_parked();
    // p0 has a report: the brain mark and the time, no words. p1 has none:
    // the hammer alone, nothing after it.
    assert_eq!(draw(cx), [true, false, false]);
    let (sep, tab) = (
        cx.debug_bounds("agent-sep-agent-p0").unwrap(),
        cx.debug_bounds("agent-tab-agent-p0").unwrap(),
    );
    let status = cx.debug_bounds("status-agent-p0").unwrap();
    for key in ["status-icon-agent-p0", "status-icon-agent-p1"] {
        let icon = cx.debug_bounds(key).unwrap();
        assert!(
            icon.size.width >= px(11.) && icon.size.width <= px(12.),
            "{icon:?}"
        );
        assert_eq!(icon.size.width, icon.size.height);
        assert!(icon.left() >= tab.right() || key.ends_with("p1"), "{key}");
    }
    let icon = cx.debug_bounds("status-icon-agent-p0").unwrap();
    assert!(icon.right() <= status.left());
    assert_eq!(icon.center().y, status.center().y);
    assert!(cx.debug_bounds("status-agent-p1").is_none());
    assert!(sep.right() <= tab.left());
    // The one-line rows keep the words, and a custom layout shows no mark.
    view.update(cx, |view, cx| {
        view.config.layout.mode = crate::config::LayoutMode::Minimal;
        cx.notify();
    });
    assert_eq!(draw(cx), [false, true, true]);
    assert!(cx.debug_bounds("status-icon-agent-p0").is_none());
    view.update(cx, |view, cx| {
        view.config.layout.mode =
            crate::config::LayoutMode::from(crate::config::Density::Comfortable);
        view.config.sidebar_layout.agents = toml::from_str("rows = [[\"state_text\"]]").unwrap();
        view.config.usage.inline = true;
        cx.notify();
    });
    draw(cx);
    assert!(cx.debug_bounds("status-icon-agent-p0").is_none());
    std::fs::remove_dir_all(&dir).ok();
}
