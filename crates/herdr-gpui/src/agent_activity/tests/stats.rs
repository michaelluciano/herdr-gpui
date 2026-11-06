use super::{Dir, counted, poll};
use crate::HerdrWindow;
use gpui::{Entity, VisualTestContext};
use std::time::Instant;

fn line(model: &str, context: u64) -> String {
    format!(
        r#"{{"type":"assistant","message":{{"model":"{model}","usage":{{"input_tokens":{context},"output_tokens":0}}}}}}"#
    )
}

fn state_with_transcript(dir: &Dir, pane: &str, transcript: &std::path::Path) {
    let since = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    std::fs::write(
        dir.path().join(format!("{pane}.json")),
        format!(
            r#"{{"state":"working","since":{since},"transcript":{}}}"#,
            serde_json::to_string(&transcript.to_string_lossy()).unwrap()
        ),
    )
    .unwrap();
}

fn stats_of(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    pane: &str,
) -> Option<(String, u64)> {
    view.read_with(cx, |view, _| {
        view.agent_activity
            .stats(pane)
            .map(|stats| (stats.model.clone(), stats.context))
    })
}

#[gpui::test]
fn a_refresh_reads_the_transcript_tail_and_a_missing_one_gives_none(cx: &mut gpui::TestAppContext) {
    let dir = Dir::new("stats");
    let transcript = dir.path().join("session.jsonl");
    // Over 256 KiB: the head is skipped, so the read starts mid-line.
    let filler = line("claude-haiku-4", 1);
    let mut text = String::new();
    while text.len() < 300_000 {
        text.push_str(&filler);
        text.push('\n');
    }
    text.push_str(&line("claude-opus-5", 117_300));
    text.push('\n');
    std::fs::write(&transcript, text).unwrap();
    state_with_transcript(&dir, "p0", &transcript);
    state_with_transcript(&dir, "p1", &dir.path().join("gone.jsonl"));
    let (view, _notified, cx) = counted(cx);
    poll(&view, cx, Instant::now(), &dir);
    assert_eq!(
        stats_of(&view, cx, "p0"),
        Some(("claude-opus-5".into(), 117_300))
    );
    assert_eq!(stats_of(&view, cx, "p1"), None);
}

#[gpui::test]
fn the_compact_window_comes_from_the_settings_file(cx: &mut gpui::TestAppContext) {
    let dir = Dir::new("limit");
    let limit = |view: &Entity<HerdrWindow>, cx: &mut VisualTestContext| {
        view.read_with(cx, |view, _| view.agent_activity.compact_limit())
    };
    let (view, _notified, cx) = counted(cx);
    let t0 = Instant::now();
    poll(&view, cx, t0, &dir);
    assert_eq!(limit(&view, cx), None, "absent file");
    std::fs::write(
        dir.path().join("settings.json"),
        r#"{"autoCompactWindow":250000,"x":1}"#,
    )
    .unwrap();
    poll(&view, cx, t0 + std::time::Duration::from_secs(3), &dir);
    assert_eq!(limit(&view, cx), Some(250_000));
    std::fs::write(dir.path().join("settings.json"), "not json").unwrap();
    poll(&view, cx, t0 + std::time::Duration::from_secs(6), &dir);
    assert_eq!(limit(&view, cx), None, "invalid file");
}
