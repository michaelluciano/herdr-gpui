use super::apply;
use super::{apply, moved};
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

fn ids(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|id| (*id).to_owned()).collect()
}

#[test]
fn moving_a_row_down_lands_it_before_the_unit_at_the_slot() {
    let order = ids(&["a", "b", "c", "d"]);
    assert_eq!(moved(&order, "a", 3), Some(ids(&["b", "c", "a", "d"])));
}

#[test]
fn moving_a_row_up_lands_it_before_the_unit_at_the_slot() {
    let order = ids(&["a", "b", "c", "d"]);
    assert_eq!(moved(&order, "d", 1), Some(ids(&["a", "d", "b", "c"])));
}

#[test]
fn the_end_slot_puts_a_row_last() {
    let order = ids(&["a", "b", "c"]);
    assert_eq!(moved(&order, "a", order.len()), Some(ids(&["b", "c", "a"])));
}

#[test]
fn dropping_a_row_into_its_own_place_moves_nothing() {
    let order = ids(&["a", "b", "c"]);
    // Its own gap, and the gap right after it.
    assert_eq!(moved(&order, "b", 1), None);
    assert_eq!(moved(&order, "b", 2), None);
    assert_eq!(moved(&order, "c", order.len()), None);
}

#[test]
fn an_unlisted_row_moves_nothing() {
    assert_eq!(moved(&ids(&["a", "b"]), "z", 0), None);
}
