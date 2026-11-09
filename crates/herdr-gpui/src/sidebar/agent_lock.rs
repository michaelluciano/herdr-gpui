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

/// `order` with `dragged` lifted out and dropped into the gap `slot`, a gap
/// index in `order` as it stands (`order.len()` is the end). `None` when
/// `dragged` is not listed or the drop leaves the order as it was.
pub(super) fn moved(order: &[String], dragged: &str, slot: usize) -> Option<Vec<String>> {
    let from = order.iter().position(|id| id == dragged)?;
    let mut next = order.to_vec();
    next.remove(from);
    // The gap indices count the dragged row still in place.
    next.insert(
        if slot > from { slot - 1 } else { slot }.min(next.len()),
        dragged.to_owned(),
    );
    (next != order).then_some(next)
}

#[cfg(test)]
mod tests;
