use super::apply;
use crate::sidebar::layout_tests;
use herdr_client::protocol::ClientShellAgent;

/// Agents named by pane, copied from the layout fixture's first agent.
fn agents(panes: &[&str]) -> Vec<ClientShellAgent> {
    let template = layout_tests::snapshot(1).agents[0].clone();
    panes
        .iter()
        .map(|pane| ClientShellAgent {
            pane_id: (*pane).into(),
            ..template.clone()
        })
        .collect()
}

fn order(agents: &[ClientShellAgent], frozen: &[&str]) -> Vec<String> {
    let frozen: Vec<String> = frozen.iter().map(|id| (*id).to_owned()).collect();
    apply(agents.iter().enumerate().collect(), &frozen)
        .into_iter()
        .map(|(_, agent)| agent.pane_id.clone())
        .collect()
}

#[test]
fn the_frozen_order_wins_over_the_incoming_order() {
    let rows = agents(&["a", "b", "c"]);
    assert_eq!(order(&rows, &["c", "a", "b"]), ["c", "a", "b"]);
}

#[test]
fn unknown_agents_follow_in_incoming_order() {
    let rows = agents(&["x", "b", "y", "a"]);
    assert_eq!(order(&rows, &["a", "b"]), ["a", "b", "x", "y"]);
}

#[test]
fn an_empty_freeze_keeps_the_incoming_order() {
    let rows = agents(&["c", "a", "b"]);
    assert_eq!(order(&rows, &[]), ["c", "a", "b"]);
}

#[test]
fn frozen_agents_missing_from_the_rows_are_ignored() {
    let rows = agents(&["b", "a"]);
    assert_eq!(order(&rows, &["gone", "a", "also-gone", "b"]), ["a", "b"]);
}
