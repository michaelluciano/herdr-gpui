//! The graph view's model: loaded commits, their lane layout, the selected
//! commit, and the bookkeeping that drops results of superseded loads. Holds
//! no GPUI types; the view reads it while rendering and applies worker
//! results through [`GraphState::apply`].

use herdr_vcs::graph::{Change, Commit, Layout, Ref, layout};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

/// The files one commit changed, as loaded for the detail pane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GraphFiles {
    pub(crate) commit: String,
    pub(crate) changes: Vec<Change>,
}

#[derive(Debug, Default)]
pub(crate) struct GraphState {
    /// Set to stop the load a newer request replaced.
    pub(crate) cancel: Arc<AtomicBool>,
    pub(crate) commits: Vec<Commit>,
    /// Changed files of the selected commit; `None` until they arrive.
    pub(crate) files: Option<GraphFiles>,
    /// When the newest load began; `None` before the first.
    pub(crate) last_load: Option<Instant>,
    /// Id of the newest files load.
    pub(crate) files_request: u64,
    pub(crate) files_cancel: Arc<AtomicBool>,
    /// The checked-out local branch; `None` when HEAD is detached.
    pub(crate) head_branch: Option<String>,
    pub(crate) layout: Layout,
    /// Id of the newest load; results of older ones are stale.
    pub(crate) request: u64,
    /// Selected commit id, kept across refreshes while the commit exists.
    pub(crate) selected: Option<String>,
    /// Row index the list should scroll into view, taken by the view.
    pub(crate) scroll_target: Option<usize>,
}

impl GraphState {
    /// Start a files load for `commit`: cancels the previous one and returns
    /// the id and cancel flag to hand to the worker.
    pub(crate) fn begin_files_request(&mut self) -> (u64, Arc<AtomicBool>) {
        self.files_cancel.store(true, Ordering::SeqCst);
        self.files_request = self.files_request.wrapping_add(1);
        self.files_cancel = Arc::new(AtomicBool::new(false));
        (self.files_request, Arc::clone(&self.files_cancel))
    }

    /// Store a finished files load. Returns whether anything changed: stale
    /// requests, commits that are not selected and unchanged lists change
    /// nothing.
    pub(crate) fn apply_files(&mut self, request: u64, commit: &str, changes: Vec<Change>) -> bool {
        if request != self.files_request || self.selected.as_deref() != Some(commit) {
            return false;
        }
        if self
            .files
            .as_ref()
            .is_some_and(|files| files.commit == commit && files.changes == changes)
        {
            return false;
        }
        self.files = Some(GraphFiles {
            commit: commit.to_owned(),
            changes,
        });
        true
    }

    /// Whether the commits are due a reload: never loaded, or `interval` has
    /// passed since the last attempt.
    pub(crate) fn refresh_due(&self, now: Instant, interval: Duration) -> bool {
        self.last_load
            .is_none_or(|last| now.saturating_duration_since(last) >= interval)
    }

    pub(crate) fn mark_loaded(&mut self, now: Instant) {
        self.last_load = Some(now);
    }

    /// Store a finished load. Returns whether anything changed: stale
    /// requests and unchanged commit lists change nothing.
    pub(crate) fn apply(&mut self, request: u64, commits: Vec<Commit>) -> bool {
        if request != self.request || commits == self.commits {
            return false;
        }
        self.layout = layout(&commits);
        self.head_branch = head_branch(&commits);
        let previous = self.selected.clone();
        self.selected = self
            .selected
            .take()
            .filter(|id| commits.iter().any(|commit| commit.id == *id))
            .or_else(|| commits.first().map(|commit| commit.id.clone()));
        if self.selected != previous {
            self.files = None;
        }
        self.commits = commits;
        if self.selected != previous {
            self.scroll_target = self.selected_index();
        }
        true
    }

    /// Start a new load: cancels the previous one and returns the id and the
    /// cancel flag to hand to the worker.
    pub(crate) fn begin_request(&mut self) -> (u64, Arc<AtomicBool>) {
        self.cancel_all();
        self.mark_loaded(Instant::now());
        self.request = self.request.wrapping_add(1);
        self.cancel = Arc::new(AtomicBool::new(false));
        (self.request, Arc::clone(&self.cancel))
    }

    pub(crate) fn cancel_all(&mut self) {
        self.cancel.store(true, Ordering::SeqCst);
        self.files_cancel.store(true, Ordering::SeqCst);
    }

    pub(crate) fn row_count(&self) -> usize {
        self.commits.len()
    }

    /// Select a loaded commit. Returns whether the selection changed.
    pub(crate) fn select(&mut self, id: &str) -> bool {
        if self.selected.as_deref() == Some(id) || !self.commits.iter().any(|c| c.id == id) {
            return false;
        }
        self.selected = Some(id.to_owned());
        self.files = None;
        self.scroll_target = self.selected_index();
        true
    }

    /// Take the row to scroll into view, if the selection moved.
    pub(crate) fn take_scroll_target(&mut self) -> Option<usize> {
        self.scroll_target.take()
    }

    pub(crate) fn selected_index(&self) -> Option<usize> {
        let id = self.selected.as_deref()?;
        self.commits.iter().position(|commit| commit.id == id)
    }
}

fn head_branch(commits: &[Commit]) -> Option<String> {
    let head = commits
        .iter()
        .find(|commit| commit.refs.contains(&Ref::Head))?;
    head.refs.iter().find_map(|reference| match reference {
        Ref::LocalBranch(name) => Some(name.clone()),
        _ => None,
    })
}
