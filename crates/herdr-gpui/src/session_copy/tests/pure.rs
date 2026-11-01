use super::*;
use crate::session_copy::{Session, of_pane, of_workspace};

#[test]
fn workspace_with_two_agents_has_two_labelled_entries() {
    let entries = of_workspace(&snapshot(), &two_agents(), "w0");
    let labels: Vec<String> = entries.iter().map(Session::label).collect();
    assert_eq!(labels, ["claude \u{b7} 184567ac", "codex \u{b7} 9f3c2b1a"]);
}

#[test]
fn copied_text_is_the_full_session_id() {
    let entry = of_pane(&two_agents(), "p0").unwrap();
    assert_eq!(entry.id, CLAUDE_ID);
    assert_eq!(entry.agent, "claude");
}

#[test]
fn no_entry_without_a_session_id() {
    assert!(of_pane(&[], "p0").is_none());
    assert!(of_pane(&two_agents(), "p1").is_none());
    assert!(of_workspace(&snapshot(), &[], "w0").is_empty());
    // A session in another workspace's pane does not show under w0.
    let elsewhere = [binding("p9", Agent::Claude, CLAUDE_ID)];
    assert!(of_workspace(&snapshot(), &elsewhere, "w0").is_empty());
}
