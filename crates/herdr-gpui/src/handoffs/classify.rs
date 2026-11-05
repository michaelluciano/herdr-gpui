//! Sorts the loaded handoffs into the active list and the history.

use super::list::Handoff;
use std::collections::HashSet;

/// Why a handoff is in the history instead of the active list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shelf {
    /// Claimed or consumed, or somebody holds its claim.
    PickedUp,
    /// A newer handoff of its project exists.
    Superseded,
}

impl Shelf {
    /// The words a history row's second line ends with.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::PickedUp => "picked up",
            Self::Superseded => "superseded",
        }
    }
}

/// A handoff in the history and why it is there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Shelved {
    pub(crate) shelf: Shelf,
    pub(crate) handoff: Handoff,
}

/// What the sidebar shows: the active handoffs, then the history. Each list
/// keeps the order it was given.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Board {
    pub(crate) active: Vec<Handoff>,
    pub(crate) history: Vec<Shelved>,
}

/// Sorts `list`, as [`super::list::list_all`] returns it (newest first), into
/// the board. Per project only the newest handoff can be active, and only
/// while it is open and unclaimed; every older one is superseded. A handoff
/// whose status is neither open nor picked up is hidden and does not count as
/// the project's newest.
pub(crate) fn classify(list: Vec<Handoff>) -> Board {
    let mut seen: HashSet<String> = HashSet::new();
    let mut board = Board::default();
    for handoff in list {
        let picked_up = match handoff.status.as_str() {
            "claimed" | "consumed" => true,
            // A claim on an open handoff picks it up; a claim on any other
            // status means nothing to the sidebar.
            "open" => !handoff.claimed_by.is_empty(),
            _ => continue,
        };
        let newest = seen.insert(handoff.project.clone());
        match (newest, picked_up) {
            (true, false) => board.active.push(handoff),
            (true, true) => board.history.push(Shelved {
                shelf: Shelf::PickedUp,
                handoff,
            }),
            (false, _) => board.history.push(Shelved {
                shelf: Shelf::Superseded,
                handoff,
            }),
        }
    }
    board
}
