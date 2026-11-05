use super::*;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// A Claude agent's row shows what its hooks report after the name, in the
/// status word's column; an agent without a report shows the daemon's word.
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
            ["thinking · 2m", "working"].map(|text| probes.contains_key(text))
        })
    };
    // Before the first read the default row shows no status word.
    assert_eq!(draw(cx), [false, false]);
    let path = dir.clone();
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.poll_agent_activity_at(Instant::now(), path, cx)
        })
    });
    cx.run_until_parked();
    // p0 has a report; p1 has none and keeps the daemon's word.
    assert_eq!(draw(cx), [true, true]);
    std::fs::remove_dir_all(&dir).ok();
}
