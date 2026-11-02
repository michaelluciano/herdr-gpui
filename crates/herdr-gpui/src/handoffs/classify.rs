//! Sorts the loaded handoffs into the active list and the history.

use super::list::Handoff;

/// Why a handoff is in the history instead of the active list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shelf {
    /// Claimed or consumed, or somebody holds its claim.
    PickedUp,
    /// Another handoff of its project names it as `prev`, or is newer in the
    /// same session.
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

/// How one loaded handoff is shown.
enum Fate {
    Active,
    Shelved(Shelf),
    /// Not shown: a status that is neither open nor picked up.
    Hidden,
}

/// Sorts `list`, as [`super::list::list_all`] returns it, into the board.
pub(crate) fn classify(list: Vec<Handoff>) -> Board {
    let fates: Vec<Fate> = (0..list.len()).map(|index| fate(index, &list)).collect();
    let mut board = Board::default();
    for (fate, handoff) in fates.into_iter().zip(list) {
        match fate {
            Fate::Active => board.active.push(handoff),
            Fate::Shelved(shelf) => board.history.push(Shelved { shelf, handoff }),
            Fate::Hidden => {}
        }
    }
    board
}

fn fate(index: usize, list: &[Handoff]) -> Fate {
    let handoff = &list[index];
    match handoff.status.as_str() {
        "claimed" | "consumed" => Fate::Shelved(Shelf::PickedUp),
        // A claim on an open handoff picks it up; a claim on any other
        // status means nothing to the sidebar.
        "open" if !handoff.claimed_by.is_empty() => Fate::Shelved(Shelf::PickedUp),
        "open" if superseded(index, list) => Fate::Shelved(Shelf::Superseded),
        "open" => Fate::Active,
        _ => Fate::Hidden,
    }
}

/// Whether another handoff of the same project names this one as `prev`, or
/// is newer in the same non-empty session. Any status counts as "another":
/// a claim that picks a handoff up is how it gets superseded.
fn superseded(index: usize, list: &[Handoff]) -> bool {
    let handoff = &list[index];
    list.iter().enumerate().any(|(other, later)| {
        other != index
            && later.project == handoff.project
            && ((!later.prev.is_empty() && later.prev == handoff.id)
                || (!handoff.session_id.is_empty()
                    && later.session_id == handoff.session_id
                    && newer(later, handoff)))
    })
}

/// Whether `a` sorts before `b` in the loader's order: later `created`
/// first, then the path. Exactly one handoff of a session is the newest.
fn newer(a: &Handoff, b: &Handoff) -> bool {
    a.created > b.created || (a.created == b.created && a.path < b.path)
}

/// The newest of `active` (newest first, as the board keeps it) that was
/// written for the focused workspace: on its `branch`, inside its checkout
/// `cwd`, or both when the handoff names both. A handoff naming neither
/// belongs to no workspace.
pub(crate) fn current<'a>(
    active: &'a [Handoff],
    branch: Option<&str>,
    cwd: &str,
) -> Option<&'a Handoff> {
    active.iter().find(|handoff| {
        let on_branch = || branch.is_some_and(|branch| branch == handoff.branch);
        let in_checkout = || !cwd.is_empty() && std::path::Path::new(&handoff.cwd).starts_with(cwd);
        match (handoff.branch.is_empty(), handoff.cwd.is_empty()) {
            (true, true) => false,
            (false, true) => on_branch(),
            (true, false) => in_checkout(),
            (false, false) => on_branch() && in_checkout(),
        }
    })
}
