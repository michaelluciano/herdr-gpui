//! The refresh of the handoff board: a one second check of the root, and a
//! full read when it changed or five seconds passed.

use super::{
    classify::{Board, classify_with},
    dismissed,
    list::{Handoff, list_all},
    pinned,
    signature::signature,
};
use crate::HerdrWindow;
use crate::state_file;
use gpui::Context;
use std::{
    collections::HashSet,
    fmt,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

/// The longest the handoff directory goes unread.
pub(crate) const HANDOFF_REFRESH: Duration = Duration::from_secs(5);

/// How often the root's fingerprint is compared; a change reads it at once.
pub(crate) const HANDOFF_CHECK: Duration = Duration::from_secs(1);

/// The window's handoff board and when it was last read.
#[derive(Debug, Default)]
pub(crate) struct Handoffs {
    pub(crate) board: Board,
    /// The handoffs behind `board`, as the last read returned them.
    list: Vec<Handoff>,
    /// Files removed from the list; saved when `saver` is set.
    pub(crate) dismissed: HashSet<PathBuf>,
    /// Files pinned to the top of the active list; saved when `pin_saver` is set.
    pub(crate) pinned: HashSet<PathBuf>,
    saver: Option<Saver>,
    pin_saver: Option<Saver>,
    /// The saved set was read.
    restored: bool,
    /// When the list was last read in full.
    pub(crate) last_poll: Option<Instant>,
    /// When the root was last fingerprinted.
    last_check: Option<Instant>,
    /// The root's fingerprint at the last check.
    fingerprint: Option<u64>,
    /// The section body is folded; its header stays.
    pub(crate) collapsed: bool,
    /// The history rows show below the active rows.
    pub(crate) history_open: bool,
    /// A read is running, so ticks do not stack another.
    loading: bool,
}

/// Saves a path set off the UI thread.
struct Saver(state_file::Writer<Vec<String>>);

impl fmt::Debug for Saver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Saver")
    }
}

impl Handoffs {
    /// Replaces the handoffs and sorts them against the dismissed set.
    pub(crate) fn set_list(&mut self, list: Vec<Handoff>) {
        self.list = list;
        self.rebuild();
    }

    fn rebuild(&mut self) {
        self.board = classify_with(self.list.clone(), &self.dismissed).pinned_first(&self.pinned);
    }

    /// Removes one handoff from the list.
    pub(crate) fn dismiss(&mut self, path: PathBuf) {
        if self.dismissed.insert(path) {
            self.changed();
        }
    }

    /// Dismisses every picked-up row; superseded rows stay.
    pub(crate) fn clear_picked_up(&mut self) {
        let paths = self.board.picked_up_paths();
        if !paths.is_empty() {
            self.dismissed.extend(paths);
            self.changed();
        }
    }

    /// Forgets every dismissal.
    pub(crate) fn restore_dismissed(&mut self) {
        if !self.dismissed.is_empty() {
            self.dismissed.clear();
            self.changed();
        }
    }

    /// Pins `path`, or unpins it when pinned. Pins order the active list only.
    pub(crate) fn toggle_pin(&mut self, path: PathBuf) {
        if !self.pinned.remove(&path) {
            self.pinned.insert(path);
        }
        self.rebuild();
        if let Some(saver) = &self.pin_saver {
            saver.0.save(pinned::to_saved(&self.pinned));
        }
    }

    fn changed(&mut self) {
        self.rebuild();
        if let Some(saver) = &self.saver {
            saver.0.save(dismissed::to_saved(&self.dismissed));
        }
    }

    /// Reads the saved set and starts its writer, once. Real polls only;
    /// fixtures never save.
    fn restore(&mut self) {
        if std::mem::replace(&mut self.restored, true) {
            return;
        }
        if let Some(path) = dismissed::file() {
            self.dismissed.extend(dismissed::load(&path));
            match state_file::Writer::start("handoff-dismissed", path) {
                Ok(writer) => self.saver = Some(Saver(writer)),
                Err(error) => tracing::warn!(%error, "Cannot start the dismissed handoffs worker"),
            }
        }
        if let Some(path) = pinned::file() {
            self.pinned.extend(pinned::load(&path));
            match state_file::Writer::start("handoff-pinned", path) {
                Ok(writer) => self.pin_saver = Some(Saver(writer)),
                Err(error) => tracing::warn!(%error, "Cannot start the pinned handoffs worker"),
            }
        }
        self.rebuild();
    }
}

/// The handoffs root under a home directory.
pub(crate) fn root_under(home: &Path) -> PathBuf {
    home.join("claude-private").join("handoffs")
}

/// The root under `$HOME`, `None` without one.
pub(crate) fn handoffs_root() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(|home| root_under(Path::new(&home)))
}

impl HerdrWindow {
    /// Runs every window tick; reads the real root when a read is due.
    pub(crate) fn poll_handoffs(&mut self, cx: &mut Context<Self>) {
        if let Some(root) = handoffs_root() {
            self.handoffs.restore();
            self.poll_handoffs_at(Instant::now(), root, cx);
        }
    }

    /// Fingerprints `root` off the UI thread once a second, and lists it when
    /// the fingerprint moved or the last full read is five seconds old at
    /// `now`. Notifies only when the list changed.
    pub(crate) fn poll_handoffs_at(&mut self, now: Instant, root: PathBuf, cx: &mut Context<Self>) {
        let checked = self.handoffs.last_check;
        if self.handoffs.loading
            || checked.is_some_and(|at| now.saturating_duration_since(at) < HANDOFF_CHECK)
        {
            return;
        }
        let stale = self
            .handoffs
            .last_poll
            .is_none_or(|at| now.saturating_duration_since(at) >= HANDOFF_REFRESH);
        let known = self.handoffs.fingerprint;
        self.handoffs.last_check = Some(now);
        self.handoffs.loading = true;
        let reading = cx.background_executor().spawn(async move {
            let current = signature(&root);
            let moved = known != Some(current);
            ((stale || moved).then(|| list_all(&root)), current)
        });
        cx.spawn(async move |this, cx| {
            let (list, current) = reading.await;
            this.update(cx, |this, cx| {
                this.handoffs.loading = false;
                this.handoffs.fingerprint = Some(current);
                let Some(list) = list else { return };
                this.handoffs.last_poll = Some(now);
                let before = std::mem::take(&mut this.handoffs.board);
                this.handoffs.set_list(list);
                if this.handoffs.board != before {
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }
}
