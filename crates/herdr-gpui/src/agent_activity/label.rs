//! What a Claude agent's row says it is doing: a per-pane report from Claude
//! Code's hooks, checked against the daemon's own status.

use herdr_client::protocol::AgentStatus;

/// A report older than this while the daemon says working means a hook
/// missed a Stop, so the report is not trusted.
const STALE: u64 = 15 * 60;
/// A waiting report older than this reads as idle.
const IDLE_AFTER: u64 = 10 * 60;

/// What a report says the agent is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReportState {
    Thinking,
    Working,
    Subagents,
    Waiting,
}

/// One pane's parsed report. Untrusted: the reader bounds every field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Report {
    pub(crate) state: ReportState,
    /// Unix epoch seconds the state began.
    pub(crate) since: u64,
    pub(crate) subagents: u32,
    pub(crate) tool: String,
}

/// Which theme color a label takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tone {
    /// The working color.
    Working,
    /// The theme accent.
    Subagents,
    /// The done color: the agent waits for its person.
    Waiting,
    /// The blocked color: a prompt is on screen.
    Blocked,
    /// The idle color.
    Muted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Label {
    pub(crate) text: String,
    pub(crate) tone: Tone,
}

fn label(text: impl Into<String>, tone: Tone) -> Label {
    Label {
        text: text.into(),
        tone,
    }
}

/// `12s`, `3m` or `1h`.
fn elapsed(seconds: u64) -> String {
    match seconds {
        0..=59 => format!("{seconds}s"),
        60..=3599 => format!("{}m", seconds / 60),
        _ => format!("{}h", seconds / 3600),
    }
}

/// The words for the daemon's status alone.
fn from_daemon(status: AgentStatus) -> Option<Label> {
    Some(match status {
        AgentStatus::Working => label("working", Tone::Working),
        AgentStatus::Blocked => label("waiting on you", Tone::Blocked),
        AgentStatus::Done => label("waiting on you", Tone::Waiting),
        AgentStatus::Idle => label("idle", Tone::Muted),
        AgentStatus::Unknown => return None,
    })
}

/// The label for a Claude agent's row at epoch second `now`, or None when
/// nothing is known. A blocked daemon status always wins; a report the daemon
/// contradicts or that went stale gives way to the daemon's status.
pub(crate) fn activity_label(
    report: Option<&Report>,
    status: AgentStatus,
    now: u64,
) -> Option<Label> {
    let Some(report) = report else {
        return from_daemon(status);
    };
    let age = now.saturating_sub(report.since);
    let live = report.state != ReportState::Waiting;
    if status == AgentStatus::Blocked
        || (status == AgentStatus::Working && age > STALE)
        || (live && matches!(status, AgentStatus::Idle | AgentStatus::Done))
    {
        return from_daemon(status);
    }
    let since = elapsed(age);
    Some(match report.state {
        ReportState::Thinking => label(format!("thinking · {since}"), Tone::Working),
        ReportState::Working if report.tool.is_empty() => {
            label(format!("working · {since}"), Tone::Working)
        }
        ReportState::Working => label(
            format!("working · {} · {since}", report.tool),
            Tone::Working,
        ),
        ReportState::Subagents => {
            let text = match report.subagents {
                0 => "subagents".to_owned(),
                n => format!("subagents ×{n}"),
            };
            label(format!("{text} · {since}"), Tone::Subagents)
        }
        ReportState::Waiting if age > IDLE_AFTER => label("idle", Tone::Muted),
        ReportState::Waiting => label(format!("waiting on you · {since}"), Tone::Waiting),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const NOW: u64 = 10_000;

    fn report(state: ReportState, age: u64) -> Report {
        Report {
            state,
            since: NOW - age,
            subagents: 0,
            tool: String::new(),
        }
    }

    fn text(report: Option<&Report>, status: AgentStatus) -> Option<String> {
        activity_label(report, status, NOW).map(|label| label.text)
    }

    #[test]
    fn each_state_has_its_words_and_tone() {
        let w = AgentStatus::Working;
        let l = activity_label(Some(&report(ReportState::Thinking, 12)), w, NOW).unwrap();
        assert_eq!((l.text.as_str(), l.tone), ("thinking · 12s", Tone::Working));
        let mut r = report(ReportState::Working, 180);
        assert_eq!(text(Some(&r), w).unwrap(), "working · 3m");
        r.tool = "Bash".into();
        assert_eq!(text(Some(&r), w).unwrap(), "working · Bash · 3m");
        let mut r = report(ReportState::Subagents, 840);
        r.subagents = 3;
        let l = activity_label(Some(&r), w, NOW).unwrap();
        assert_eq!(
            (l.text.as_str(), l.tone),
            ("subagents ×3 · 14m", Tone::Subagents)
        );
        r.subagents = 0;
        assert_eq!(text(Some(&r), w).unwrap(), "subagents · 14m");
        let l = activity_label(
            Some(&report(ReportState::Waiting, 5)),
            AgentStatus::Done,
            NOW,
        )
        .unwrap();
        assert_eq!(
            (l.text.as_str(), l.tone),
            ("waiting on you · 5s", Tone::Waiting)
        );
    }

    #[test]
    fn waiting_turns_idle_after_ten_minutes() {
        let r = report(ReportState::Waiting, 601);
        let l = activity_label(Some(&r), AgentStatus::Idle, NOW).unwrap();
        assert_eq!((l.text.as_str(), l.tone), ("idle", Tone::Muted));
        let r = report(ReportState::Waiting, 600);
        assert_eq!(
            text(Some(&r), AgentStatus::Idle).unwrap(),
            "waiting on you · 10m"
        );
    }

    #[test]
    fn no_report_falls_back_to_the_daemon_status() {
        assert_eq!(text(None, AgentStatus::Working).unwrap(), "working");
        assert_eq!(text(None, AgentStatus::Blocked).unwrap(), "waiting on you");
        assert_eq!(text(None, AgentStatus::Done).unwrap(), "waiting on you");
        assert_eq!(text(None, AgentStatus::Idle).unwrap(), "idle");
        assert_eq!(text(None, AgentStatus::Unknown), None);
    }

    #[test]
    fn a_stale_report_while_the_daemon_works_falls_back() {
        let r = report(ReportState::Working, 15 * 60 + 1);
        assert_eq!(text(Some(&r), AgentStatus::Working).unwrap(), "working");
        let r = report(ReportState::Working, 15 * 60);
        assert_eq!(
            text(Some(&r), AgentStatus::Working).unwrap(),
            "working · 15m"
        );
    }

    #[test]
    fn a_busy_report_the_daemon_contradicts_falls_back() {
        for state in [
            ReportState::Thinking,
            ReportState::Working,
            ReportState::Subagents,
        ] {
            let r = report(state, 5);
            assert_eq!(text(Some(&r), AgentStatus::Idle).unwrap(), "idle");
            assert_eq!(text(Some(&r), AgentStatus::Done).unwrap(), "waiting on you");
        }
    }

    #[test]
    fn a_blocked_daemon_always_wins() {
        for state in [
            ReportState::Thinking,
            ReportState::Working,
            ReportState::Subagents,
            ReportState::Waiting,
        ] {
            let l = activity_label(Some(&report(state, 5)), AgentStatus::Blocked, NOW).unwrap();
            assert_eq!((l.text.as_str(), l.tone), ("waiting on you", Tone::Blocked));
        }
    }

    #[test]
    fn a_future_since_counts_as_zero_seconds() {
        let r = Report {
            since: NOW + 50,
            ..report(ReportState::Thinking, 0)
        };
        assert_eq!(
            text(Some(&r), AgentStatus::Working).unwrap(),
            "thinking · 0s"
        );
    }
}
