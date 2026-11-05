//! Sorts the loaded handoffs into the active list and the history.

use super::list::Handoff;
use std::{collections::HashSet, path::PathBuf};

/// Why a handoff is in the history instead of the active list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shelf {
    /// Claimed or consumed, or somebody holds its claim.
    PickedUp,
    /// A newer handoff of its project exists.
    Superseded,
    /// The operator removed it from the list.
    Dismissed,
}

impl Shelf {
    /// The words a history row's second line ends with.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::PickedUp => "picked up",
            Self::Superseded => "superseded",
            Self::Dismissed => "dismissed",
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

impl Board {
    /// How many history rows sit on `shelf`.
    pub(crate) fn count(&self, shelf: Shelf) -> usize {
        self.history.iter().filter(|s| s.shelf == shelf).count()
    }

    /// The files of the picked-up rows.
    pub(crate) fn picked_up_paths(&self) -> Vec<PathBuf> {
        self.history
            .iter()
            .filter(|s| s.shelf == Shelf::PickedUp)
            .map(|s| s.handoff.path.clone())
            .collect()
    }
}

/// [`classify_with`] with nothing dismissed.
#[cfg(test)]
pub(crate) fn classify(list: Vec<Handoff>) -> Board {
    classify_with(list, &HashSet::new())
}

/// Sorts `list`, as [`super::list::list_all`] returns it (newest first), into
/// the board. Per project only the newest handoff can be active, and only
/// while it is open and unclaimed; every older one is superseded. A handoff
/// whose status is neither open nor picked up is hidden and does not count as
/// the project's newest. A dismissed handoff still counts as its project's
/// newest, so the older ones stay superseded, but it goes to the history as
/// [`Shelf::Dismissed`].
pub(crate) fn classify_with(list: Vec<Handoff>, dismissed: &HashSet<PathBuf>) -> Board {
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
        if dismissed.contains(&handoff.path) {
            board.history.push(Shelved {
                shelf: Shelf::Dismissed,
                handoff,
            });
            continue;
        }
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
