#![allow(clippy::unwrap_used)]

use crate::HerdrWindow;
use crate::sidebar::layout_tests::{SidebarFixture, fixture_window};
use gpui::{AppContext as _, Entity, VisualTestContext, px, size};
use herdr_client::protocol::{AgentStatus, ClientShellAgent};
use std::{
    cell::Cell,
    path::{Path, PathBuf},
    rc::Rc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

mod glyph;

struct Dir(PathBuf);

impl Dir {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("herdr-activity-poll-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn report(&self, pane: &str, state: &str, age: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let since = now - age;
        std::fs::write(
            self.0.join(format!("{pane}.json")),
            format!(r#"{{"state":"{state}","since":{since},"subagents":0,"tool":""}}"#),
        )
        .unwrap();
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).ok();
    }
}

fn counted(
    cx: &mut gpui::TestAppContext,
) -> (Entity<HerdrWindow>, Rc<Cell<usize>>, &mut VisualTestContext) {
    let notified = Rc::new(Cell::new(0));
    let seen = notified.clone();
    let (fixture, cx) = cx.add_window_view(move |window, cx| {
        let view = cx.new(|cx| {
            let mut view = fixture_window(window, cx);
            view.live.snapshot = Some(std::sync::Arc::new(crate::sidebar::layout_tests::snapshot(
                2,
            )));
            view
        });
        cx.observe(&view, move |_, _, cx| {
            seen.set(seen.get() + 1);
            cx.notify();
        })
        .detach();
        SidebarFixture(view)
    });
    cx.simulate_resize(size(px(800.), px(600.)));
    let view = cx.update(|_, cx| fixture.read(cx).0.clone());
    (view, notified, cx)
}

fn poll(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, at: Instant, dir: &Dir) {
    let path = dir.path().to_owned();
    cx.update(|_, cx| view.update(cx, |view, cx| view.poll_agent_activity_at(at, path, cx)));
    cx.run_until_parked();
}

fn label(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, pane: &str) -> Option<String> {
    view.read_with(cx, |view, _| {
        let agent: &ClientShellAgent = view
            .live
            .snapshot
            .as_ref()
            .unwrap()
            .agents
            .iter()
            .find(|agent| agent.pane_id == pane)
            .unwrap();
        view.agent_activity_label(agent).map(|label| label.text)
    })
}

#[gpui::test]
fn no_label_shows_before_the_first_read(cx: &mut gpui::TestAppContext) {
    let (view, _notified, cx) = counted(cx);
    assert_eq!(label(&view, cx, "p0"), None);
}

#[gpui::test]
fn a_poll_reads_reports_off_thread_and_labels_claude_panes(cx: &mut gpui::TestAppContext) {
    let dir = Dir::new("fill");
    dir.report("p0", "thinking", 130);
    let (view, notified, cx) = counted(cx);
    let before = notified.get();
    poll(&view, cx, Instant::now(), &dir);
    assert_eq!(label(&view, cx, "p0").as_deref(), Some("thinking · 2m"));
    // No report for p1: the daemon's own status.
    assert_eq!(label(&view, cx, "p1").as_deref(), Some("working"));
    assert!(notified.get() > before, "the first read notifies");
}

#[gpui::test]
fn an_unchanged_read_stays_quiet_and_a_changed_label_notifies(cx: &mut gpui::TestAppContext) {
    let dir = Dir::new("quiet");
    dir.report("p0", "thinking", 130);
    let (view, notified, cx) = counted(cx);
    let t0 = Instant::now();
    poll(&view, cx, t0, &dir);
    let settled = notified.get();
    poll(&view, cx, t0 + Duration::from_secs(3), &dir);
    assert_eq!(notified.get(), settled, "same text, no notify");

    dir.report("p0", "waiting", 5);
    // Inside two seconds nothing is read.
    poll(&view, cx, t0 + Duration::from_secs(4), &dir);
    assert_eq!(notified.get(), settled);
    poll(&view, cx, t0 + Duration::from_secs(6), &dir);
    assert!(notified.get() > settled);
}

#[gpui::test]
fn a_hostile_state_file_is_no_report(cx: &mut gpui::TestAppContext) {
    let dir = Dir::new("hostile");
    std::fs::write(dir.path().join("p0.json"), "x".repeat(10_000)).unwrap();
    let (view, _notified, cx) = counted(cx);
    poll(&view, cx, Instant::now(), &dir);
    assert_eq!(label(&view, cx, "p0").as_deref(), Some("working"));
}

#[test]
fn only_claude_agents_are_labelled() {
    let mut agent: ClientShellAgent = serde_json::from_value(serde_json::json!({
        "pane_id": "p", "workspace_id": "w", "tab_id": "t", "name": "n",
        "display_agent": "Claude Code", "agent": "claude",
        "agent_status": "working", "state_change_seq": 0, "state_labels": [],
        "tokens": [], "focused": false
    }))
    .unwrap();
    assert!(super::poll::is_claude(&agent));
    agent.agent = Some("codex".into());
    agent.display_agent = Some("Codex".into());
    assert!(!super::poll::is_claude(&agent));
    assert_eq!(agent.agent_status, AgentStatus::Working);
}
