//! Lane assignment for a commit graph: pure, deterministic, and independent
//! of any renderer.
//!
//! Each row is one commit. A renderer draws the commit dot in `Row::lane`, a
//! line from above into the dot when `has_incoming`, a vertical line through
//! every lane in `passing`, and one connector per `Edge` from the dot down to
//! the lane its parent will occupy in the next row.

use super::parse::Commit;
use std::collections::HashSet;

/// What an edge connects its commit to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EdgeKind {
    /// The parent is not in the window (cut off by the limit), so the line
    /// leaves the bottom of the row in `to_lane` and stops.
    Dangling,
    /// A parent after the first: a merge joining another line of history.
    Merge,
    /// The first parent, which continues the commit's own line.
    Parent,
}

/// A connector from a commit dot down to the lane of one of its parents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Edge {
    /// The lane of the commit this edge leaves; always the row's lane.
    pub from_lane: usize,
    pub kind: EdgeKind,
    /// The lane the line occupies below the row.
    pub to_lane: usize,
}

/// The drawing instructions for one commit.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Row {
    /// Connectors to the commit's distinct parents, first parent first.
    pub edges: Vec<Edge>,
    /// Whether a line from an earlier row ends at this commit's dot.
    pub has_incoming: bool,
    /// The lane the commit dot sits in.
    pub lane: usize,
    /// Lanes whose lines run straight through this row, ascending.
    pub passing: Vec<usize>,
}

/// Rows in input order and the number of lanes the whole graph needs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Layout {
    /// Lanes used by any row, so a renderer can size its gutter.
    pub lane_count: usize,
    pub rows: Vec<Row>,
}

/// Assign lanes to `commits`, newest first with children before parents.
/// Parents missing from the window, or listed before their child, dangle.
pub fn layout(commits: &[Commit]) -> Layout {
    let known: HashSet<&str> = commits.iter().map(|commit| commit.id.as_str()).collect();
    let mut seen: HashSet<&str> = HashSet::new();
    // `lanes[n]` is the commit the line in lane `n` is heading for.
    let mut lanes: Vec<Option<&str>> = Vec::new();
    let mut lane_count = 0;
    let mut rows = Vec::with_capacity(commits.len());
    for commit in commits {
        seen.insert(commit.id.as_str());
        let incoming = lanes
            .iter()
            .position(|target| *target == Some(commit.id.as_str()));
        let lane = incoming.unwrap_or_else(|| free_lane(&mut lanes));
        lanes[lane] = None;

        let passing = lanes
            .iter()
            .enumerate()
            .filter_map(|(index, target)| target.map(|_| index))
            .collect();

        let mut edges = Vec::new();
        let mut distinct: Vec<&str> = Vec::new();
        for parent in &commit.parents {
            if !distinct.contains(&parent.as_str()) {
                distinct.push(parent);
            }
        }
        for (position, parent) in distinct.into_iter().enumerate() {
            let kind = if position == 0 {
                EdgeKind::Parent
            } else {
                EdgeKind::Merge
            };
            let waiting = known.contains(parent) && !seen.contains(parent);
            let (kind, to_lane) = if !waiting {
                (EdgeKind::Dangling, lane)
            } else if let Some(target) = lanes.iter().position(|target| *target == Some(parent)) {
                (kind, target)
            } else {
                // The first parent continues in this commit's own lane.
                let target = if position == 0 {
                    lane
                } else {
                    free_lane(&mut lanes)
                };
                lanes[target] = Some(parent);
                (kind, target)
            };
            edges.push(Edge {
                from_lane: lane,
                kind,
                to_lane,
            });
        }
        lane_count = lane_count.max(lanes.len());
        rows.push(Row {
            edges,
            has_incoming: incoming.is_some(),
            lane,
            passing,
        });
    }
    Layout { lane_count, rows }
}

/// The lowest empty lane, opening a new one on the right when all are taken.
fn free_lane(lanes: &mut Vec<Option<&str>>) -> usize {
    if let Some(free) = lanes.iter().position(Option::is_none) {
        return free;
    }
    lanes.push(None);
    lanes.len() - 1
}
