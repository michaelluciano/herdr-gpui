//! The agent panel's order lock: rows keep the order they had when the user
//! locked the panel instead of re-sorting as statuses change.

use herdr_client::protocol::ClientShellAgent;

/// `rows` ranked by their place in `frozen`. Rows `frozen` does not name,
/// such as agents that appeared after the lock, follow in their incoming
/// order; names with no row are ignored.
pub(super) fn apply<'a>(
    rows: Vec<(usize, &'a ClientShellAgent)>,
    frozen: &[String],
) -> Vec<(usize, &'a ClientShellAgent)> {
    let mut rows = rows;
    rows.sort_by_key(|(_, agent)| {
        frozen
            .iter()
            .position(|id| *id == agent.pane_id)
            .unwrap_or(frozen.len())
    });
    rows
}

#[cfg(test)]
mod tests;
