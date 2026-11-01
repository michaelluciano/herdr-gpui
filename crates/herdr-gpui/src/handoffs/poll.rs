//! The five second refresh of the handoff board.

use super::{classify, classify::Board, list::list_all};
use crate::HerdrWindow;
use gpui::Context;
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

/// How often the handoff directory is read.
pub(crate) const HANDOFF_REFRESH: Duration = Duration::from_secs(5);

/// The window's handoff board and when it was last read.
#[derive(Debug, Default)]
pub(crate) struct Handoffs {
    pub(crate) board: Board,
    pub(crate) last_poll: Option<Instant>,
    /// The section body is folded; its header stays.
    pub(crate) collapsed: bool,
    /// The history rows show below the active rows.
    pub(crate) history_open: bool,
    /// A read is running, so ticks do not stack another.
    loading: bool,
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
            .spawn(async move { classify(list_all(&root)) });
        cx.spawn(async move |this, cx| {
            let board = reading.await;
            this.update(cx, |this, cx| {
                this.handoffs.loading = false;
                if this.handoffs.board != board {
                    this.handoffs.board = board;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }
}
