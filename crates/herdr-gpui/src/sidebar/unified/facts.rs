//! Resolving one agent's [`Facts`] from its host's snapshot.

use super::Facts;
use crate::sidebar::agents::{agent_name, pane_name};
use herdr_client::protocol::{ClientShellAgent, ClientShellSnapshot};

/// `endpoint` scopes the worktree key, since two hosts may share a path.
/// The daemon reports no model or timestamp for an agent; a `model` token is
/// shown when a plugin sets one.
pub(in crate::sidebar) fn of(
    endpoint: &str,
    agent: &ClientShellAgent,
    snapshot: &ClientShellSnapshot,
) -> Facts {
    let name = agent_name(agent);
    let workspace = snapshot
        .workspaces
        .iter()
        .find(|workspace| workspace.workspace_id == agent.workspace_id);
    let (key, label) = match workspace {
        Some(workspace) => match &workspace.worktree {
            Some(worktree) => (worktree.key.as_str(), worktree.label.as_str()),
            None => (workspace.workspace_id.as_str(), workspace.label.as_str()),
        },
        None => (agent.workspace_id.as_str(), ""),
    };
    Facts {
        status: agent.agent_status,
        title: pane_name(agent, snapshot, name).unwrap_or_else(|| name.to_owned()),
        workspace: workspace.map(|workspace| workspace.label.clone()),
        branch: workspace.and_then(|workspace| workspace.branch.clone()),
        model: agent
            .tokens
            .iter()
            .find(|(token, _)| token == "model")
            .map(|(_, value)| value.clone()),
        group_key: format!("{endpoint}\u{1f}{key}"),
        group_label: label.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sidebar::layout_tests::snapshot;
    use herdr_client::protocol::ClientShellWorktree;

    #[test]
    fn facts_come_from_the_agent_and_its_workspace() {
        let mut snapshot = snapshot(1);
        snapshot.agents[0].workspace_id = "w0".into();
        snapshot.agents[0].title = Some("Fix the sidebar".into());
        snapshot.agents[0].tokens = vec![("model".into(), "opus".into())];
        snapshot.workspaces[0].branch = Some("feat/x".into());
        snapshot.workspaces[0].worktree = Some(ClientShellWorktree {
            key: "/repo/x".into(),
            label: "x".into(),
            is_linked_worktree: true,
        });
        let facts = of("local", &snapshot.agents[0], &snapshot);
        assert_eq!(facts.title, "Fix the sidebar");
        assert_eq!(facts.branch.as_deref(), Some("feat/x"));
        assert_eq!(facts.model.as_deref(), Some("opus"));
        assert_eq!(facts.group_label, "x");
        assert_eq!(facts.group_key, "local\u{1f}/repo/x");
        assert_eq!(
            facts.workspace.as_deref(),
            Some(snapshot.workspaces[0].label.as_str())
        );
    }

    #[test]
    fn a_workspace_without_a_worktree_groups_by_itself() {
        let mut snapshot = snapshot(1);
        snapshot.agents[0].workspace_id = "w0".into();
        snapshot.workspaces[0].worktree = None;
        snapshot.agents[0].tokens.clear();
        let facts = of("ssh:a", &snapshot.agents[0], &snapshot);
        assert_eq!(facts.group_key, "ssh:a\u{1f}w0");
        assert_eq!(facts.model, None);
    }
}
