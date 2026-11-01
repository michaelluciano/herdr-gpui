#![allow(clippy::unwrap_used)]

use crate::chat::existing::Binding;
use herdr_chat_protocol::Agent;
use herdr_client::protocol::ClientShellSnapshot;

mod pure;

pub(crate) const CLAUDE_ID: &str = "184567ac-0000-4000-8000-000000000001";
pub(crate) const CODEX_ID: &str = "9f3c2b1a-0000-4000-8000-000000000002";

/// Workspace w0 holds p0 (Claude) and p2 (Codex) and p1, a plain shell.
/// Workspace w1 is empty of panes.
pub(crate) fn snapshot() -> ClientShellSnapshot {
    let mut snapshot = crate::sidebar::layout_tests::snapshot(2);
    snapshot.panes = [("p0", "t0"), ("p1", "t0"), ("p2", "t1")]
        .into_iter()
        .map(|(pane, tab)| {
            serde_json::from_value(serde_json::json!({
                "pane_id": pane, "workspace_id": "w0", "tab_id": tab, "label": pane,
                "cwd": "/repo", "foreground_cwd": "/repo", "focused": false,
                "right_click_passthrough": false,
            }))
            .unwrap()
        })
        .collect();
    snapshot
}

pub(crate) fn binding(pane: &str, agent: Agent, session: &str) -> Binding {
    Binding {
        pane_id: pane.into(),
        agent,
        session_id: session.into(),
        herdr_session: None,
        pid: 1,
        shell_pid: 1,
        started_ticks: 1,
    }
}

/// The two agent sessions of workspace w0: Claude in p0, Codex in p2.
pub(crate) fn two_agents() -> Vec<Binding> {
    vec![
        binding("p0", Agent::Claude, CLAUDE_ID),
        binding("p2", Agent::Codex, CODEX_ID),
    ]
}
