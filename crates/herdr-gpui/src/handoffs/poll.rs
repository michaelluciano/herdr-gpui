//! The five second refresh of the handoff board.

use super::{
    classify::{Board, classify_with},
    dismissed,
    list::{Handoff, list_all},
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

/// How often the handoff directory is read.
pub(crate) const HANDOFF_REFRESH: Duration = Duration::from_secs(5);

/// The window's handoff board and when it was last read.
#[derive(Debug, Default)]
pub(crate) struct Handoffs {
    pub(crate) board: Board,
    /// The handoffs behind `board`, as the last read returned them.
    list: Vec<Handoff>,
    /// Files removed from the list; saved when `saver` is set.
    pub(crate) dismissed: HashSet<PathBuf>,
    saver: Option<Saver>,
    /// The saved set was read.
    restored: bool,
    pub(crate) last_poll: Option<Instant>,
    /// The section body is folded; its header stays.
    pub(crate) collapsed: bool,
    /// The history rows show below the active rows.
    pub(crate) history_open: bool,
    /// A read is running, so ticks do not stack another.
    loading: bool,
}

/// Saves the dismissed set off the UI thread.
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
        self.board = classify_with(self.list.clone(), &self.dismissed);
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
        let Some(path) = dismissed::file() else {
            return;
        };
        self.dismissed.extend(dismissed::load(&path));
        match state_file::Writer::start("handoff-dismissed", path) {
            Ok(writer) => self.saver = Some(Saver(writer)),
            Err(error) => tracing::warn!(%error, "Cannot start the dismissed handoffs worker"),
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

    /// Reads `root` off the UI thread when the last read is five seconds old
    /// at `now`, and notifies only when the list changed.
    pub(crate) fn poll_handoffs_at(&mut self, now: Instant, root: PathBuf, cx: &mut Context<Self>) {
        let due = self
            .handoffs
            .last_poll
            .is_none_or(|at| now.saturating_duration_since(at) >= HANDOFF_REFRESH);
        if self.handoffs.loading || !due {
            return;
        }
        self.handoffs.last_poll = Some(now);
        self.handoffs.loading = true;
        let reading = cx
            .background_executor()
            .spawn(async move { list_all(&root) });
        cx.spawn(async move |this, cx| {
            let list = reading.await;
            this.update(cx, |this, cx| {
                this.handoffs.loading = false;
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
