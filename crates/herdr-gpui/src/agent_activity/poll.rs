//! The two second refresh of the agents' state files.

use super::{
    label::{Label, Report, activity_label},
    read::{read_compact_limit, read_reports, read_stats, settings_path, state_dir},
    stats::Stats,
};
use crate::HerdrWindow;
use gpui::Context;
use herdr_client::protocol::{AgentStatus, ClientShellAgent};
use std::{
    collections::HashMap,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

/// How often the state directory is read.
pub(crate) const ACTIVITY_REFRESH: Duration = Duration::from_secs(2);

/// The window's reports of its local Claude agents and when they were read.
#[derive(Debug, Default)]
pub(crate) struct AgentActivity {
    reports: HashMap<String, Report>,
    /// The transcript stats of the panes whose report names a transcript.
    stats: HashMap<String, Stats>,
    /// The auto-compact window in the Claude settings.
    compact_limit: Option<u64>,
    /// A read finished, so labels may be shown; fixtures and a window with no
    /// `$HOME` keep the daemon's own state words.
    polled: bool,
    /// The label text each pane showed at the last read, to notify on change.
    shown: HashMap<String, String>,
    last_poll: Option<Instant>,
    /// A read is running, so ticks do not stack another.
    loading: bool,
}

/// Whether `agent` is Claude Code.
pub(crate) fn is_claude(agent: &ClientShellAgent) -> bool {
    agent
        .agent
        .as_deref()
        .is_some_and(|name| name.eq_ignore_ascii_case("claude"))
        || agent
            .display_agent
            .as_deref()
            .is_some_and(|name| name.to_ascii_lowercase().starts_with("claude"))
}

fn epoch_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

impl AgentActivity {
    /// The directory the Claude session in `pane_id` works in, from its
    /// report, which exists only for a local Claude pane.
    pub(crate) fn session_cwd(&self, pane_id: &str) -> Option<&str> {
        self.reports.get(pane_id)?.cwd.as_deref()
    }

    /// Gives `pane_id` transcript stats.
    #[cfg(test)]
    pub(crate) fn set_test_stats(&mut self, pane_id: &str, model: &str, context: u64) {
        self.stats.insert(
            pane_id.to_owned(),
            Stats {
                model: model.to_owned(),
                context,
            },
        );
    }

    /// Sets the auto-compact window.
    #[cfg(test)]
    pub(crate) fn set_test_limit(&mut self, limit: u64) {
        self.compact_limit = Some(limit);
    }

    /// Gives `pane_id` a report that names `cwd`.
    #[cfg(test)]
    pub(crate) fn set_test_cwd(&mut self, pane_id: &str, cwd: &str) {
        use super::label::ReportState;
        self.reports.insert(
            pane_id.to_owned(),
            Report {
                state: ReportState::Working,
                since: 0,
                subagents: 0,
                tool: String::new(),
                cwd: Some(cwd.to_owned()),
                transcript: None,
            },
        );
    }

    /// The transcript stats of the Claude session in `pane_id`.
    pub(crate) fn stats(&self, pane_id: &str) -> Option<&Stats> {
        self.stats.get(pane_id)
    }

    /// The auto-compact window from the Claude settings, if set.
    pub(crate) fn compact_limit(&self) -> Option<u64> {
        self.compact_limit
    }

    fn labels(
        &self,
        panes: &[(String, AgentStatus)],
        reports: &HashMap<String, Report>,
        now: u64,
    ) -> HashMap<String, String> {
        panes
            .iter()
            .filter_map(|(id, status)| {
                let label = activity_label(reports.get(id), *status, now)?;
                Some((id.clone(), label.text))
            })
            .collect()
    }

    /// The label for `agent` at epoch second `now`, once a read finished.
    pub(crate) fn label(&self, agent: &ClientShellAgent, now: u64) -> Option<Label> {
        if !self.polled || !is_claude(agent) {
            return None;
        }
        activity_label(self.reports.get(&agent.pane_id), agent.agent_status, now)
    }
}

impl HerdrWindow {
    /// The label for a Claude agent's row, `None` for other agents.
    pub(crate) fn agent_activity_label(&self, agent: &ClientShellAgent) -> Option<Label> {
        self.agent_activity.label(agent, epoch_now())
    }

    /// The local Claude agents with the daemon's status for each. Other
    /// hosts' panes have no state files on this computer.
    fn local_claude_panes(&self) -> Vec<(String, AgentStatus)> {
        let mut panes = Vec::new();
        for (index, endpoint) in self.endpoints.iter().enumerate() {
            if endpoint.id != crate::endpoint::LOCAL {
                continue;
            }
            let live = if index == self.selected_endpoint {
                &self.live
            } else {
                &endpoint.live
            };
            if let Some(snapshot) = live.snapshot.as_deref() {
                panes.extend(
                    snapshot
                        .agents
                        .iter()
                        .filter(|agent| is_claude(agent))
                        .map(|agent| (agent.pane_id.clone(), agent.agent_status)),
                );
            }
        }
        panes
    }

    /// Runs every window tick; reads the real directory when a read is due.
    pub(crate) fn poll_agent_activity(&mut self, cx: &mut Context<Self>) {
        if let Some(dir) = state_dir() {
            self.poll_agent_activity_at(Instant::now(), dir, settings_path(), cx);
        }
    }

    /// Reads `dir` off the UI thread when the last read is two seconds old at
    /// `now`, and notifies only when a pane's label text changed.
    pub(crate) fn poll_agent_activity_at(
        &mut self,
        now: Instant,
        dir: PathBuf,
        settings: Option<PathBuf>,
        cx: &mut Context<Self>,
    ) {
        let state = &mut self.agent_activity;
        let due = state
            .last_poll
            .is_none_or(|at| now.saturating_duration_since(at) >= ACTIVITY_REFRESH);
        if state.loading || !due {
            return;
        }
        state.last_poll = Some(now);
        state.loading = true;
        let panes = self.local_claude_panes();
        let ids: Vec<String> = panes.iter().map(|(id, _)| id.clone()).collect();
        let reading = cx.background_executor().spawn(async move {
            let reports = read_reports(&dir, &ids);
            let stats = read_stats(&reports);
            let limit = settings.as_deref().and_then(read_compact_limit);
            (reports, stats, limit)
        });
        cx.spawn(async move |this, cx| {
            let (reports, stats, limit) = reading.await;
            this.update(cx, |this, cx| {
                let state = &mut this.agent_activity;
                state.loading = false;
                let shown = state.labels(&panes, &reports, epoch_now());
                let changed = !state.polled || shown != state.shown;
                state.reports = reports;
                state.stats = stats;
                state.compact_limit = limit;
                state.shown = shown;
                state.polled = true;
                if changed {
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }
}
