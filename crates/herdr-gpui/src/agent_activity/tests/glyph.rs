//! The glyph and structured detail each activity label carries beside its words.
use crate::agent_activity::{
    glyph::{Glyph, tool_glyph},
    label::{Report, ReportState, activity_label},
};
use herdr_client::protocol::AgentStatus;

const NOW: u64 = 10_000;

fn report(state: ReportState, age: u64, tool: &str, subagents: u32) -> Report {
    Report {
        state,
        since: NOW - age,
        subagents,
        tool: tool.into(),
        cwd: None,
    }
}

fn of(report: Option<&Report>, status: AgentStatus) -> (Glyph, String, String) {
    let label = activity_label(report, status, NOW).unwrap();
    (label.glyph, label.detail, label.text)
}

#[test]
fn each_state_picks_its_glyph_and_keeps_only_the_useful_remainder() {
    let w = AgentStatus::Working;
    let cases = [
        (
            report(ReportState::Thinking, 12, "", 0),
            w,
            Glyph::Brain,
            "12s",
        ),
        (
            report(ReportState::Working, 180, "", 0),
            w,
            Glyph::Hammer,
            "3m",
        ),
        (
            report(ReportState::Working, 180, "Bash", 0),
            w,
            Glyph::Terminal,
            "3m",
        ),
        (
            report(ReportState::Working, 180, "Frobnicate", 0),
            w,
            Glyph::Hammer,
            "Frobnicate · 3m",
        ),
        (
            report(ReportState::Subagents, 840, "", 3),
            w,
            Glyph::Subagents,
            "×3 · 14m",
        ),
        (
            report(ReportState::Subagents, 840, "", 0),
            w,
            Glyph::Subagents,
            "14m",
        ),
        (
            report(ReportState::Waiting, 5, "", 0),
            AgentStatus::Done,
            Glyph::Waiting,
            "<1m",
        ),
        (
            report(ReportState::Waiting, 125, "", 0),
            AgentStatus::Done,
            Glyph::Waiting,
            "2m",
        ),
        (
            report(ReportState::Waiting, 601, "", 0),
            AgentStatus::Idle,
            Glyph::Sleep,
            "",
        ),
    ];
    for (report, status, glyph, detail) in cases {
        let (g, d, _) = of(Some(&report), status);
        assert_eq!((g, d.as_str()), (glyph, detail), "{report:?}");
    }
}

#[test]
fn a_blocked_daemon_shows_the_alert_glyph_whatever_the_report_says() {
    let r = report(ReportState::Thinking, 5, "", 0);
    let (glyph, detail, text) = of(Some(&r), AgentStatus::Blocked);
    assert_eq!(
        (glyph, detail.as_str(), text.as_str()),
        (Glyph::Blocked, "", "waiting on you")
    );
}

#[test]
fn the_daemons_status_alone_maps_to_a_glyph_and_keeps_the_words() {
    for (status, glyph, text) in [
        (AgentStatus::Working, Glyph::Hammer, "working"),
        (AgentStatus::Idle, Glyph::Sleep, "idle"),
        (AgentStatus::Done, Glyph::Waiting, "waiting on you"),
        (AgentStatus::Blocked, Glyph::Blocked, "waiting on you"),
    ] {
        let (g, detail, words) = of(None, status);
        assert_eq!((g, detail.as_str(), words.as_str()), (glyph, "", text));
    }
}

#[test]
fn words_are_unchanged_beside_the_glyph() {
    let r = report(ReportState::Working, 180, "Bash", 0);
    assert_eq!(of(Some(&r), AgentStatus::Working).2, "working · Bash · 3m");
}

#[test]
fn every_claude_tool_has_its_glyph_and_the_rest_fall_back() {
    let table = [
        ("Bash", Glyph::Terminal),
        ("Read", Glyph::Read),
        ("Edit", Glyph::Edit),
        ("MultiEdit", Glyph::Edit),
        ("Write", Glyph::Write),
        ("NotebookEdit", Glyph::Notebook),
        ("Grep", Glyph::Grep),
        ("Glob", Glyph::Glob),
        ("WebFetch", Glyph::WebFetch),
        ("WebSearch", Glyph::WebSearch),
        ("Task", Glyph::Subagents),
        ("Agent", Glyph::Subagents),
        ("TodoWrite", Glyph::Todo),
        ("Skill", Glyph::Skill),
        ("ExitPlanMode", Glyph::Plan),
        ("EnterPlanMode", Glyph::Plan),
        ("AskUserQuestion", Glyph::Ask),
        ("mcp__github__create_issue", Glyph::Plug),
        ("mcp__", Glyph::Plug),
        ("Unheard", Glyph::Hammer),
        ("bash", Glyph::Hammer),
        ("", Glyph::Hammer),
    ];
    for (name, glyph) in table {
        assert_eq!(tool_glyph(name), glyph, "{name:?}");
    }
}

#[test]
fn an_mcp_tool_keeps_a_short_name_and_a_known_tool_keeps_none() {
    let mcp = report(ReportState::Working, 60, "mcp__github__create_issue", 0);
    let (glyph, detail, _) = of(Some(&mcp), AgentStatus::Working);
    assert_eq!(
        (glyph, detail.as_str()),
        (Glyph::Plug, "github.create_issue · 1m")
    );
    let long = report(
        ReportState::Working,
        60,
        "mcp__a_very_long_server__and_a_long_tool_name",
        0,
    );
    let (_, detail, _) = of(Some(&long), AgentStatus::Working);
    let name = detail.split(" · ").next().unwrap();
    assert!(name.chars().count() <= 20 && name.ends_with('…'), "{name}");
    let known = report(ReportState::Working, 60, "Read", 0);
    assert_eq!(of(Some(&known), AgentStatus::Working).1, "1m");
}

#[test]
fn every_glyph_asset_is_embedded() {
    use gpui::AssetSource;
    for glyph in Glyph::ALL {
        assert!(
            crate::icons::Icons.load(glyph.path()).unwrap().is_some(),
            "{glyph:?}"
        );
    }
}
