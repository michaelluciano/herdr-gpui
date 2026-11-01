//! The five second refresh of what the launcher rows show.
use super::observe::{Observed, observe};
use crate::HerdrWindow;
use crate::config::llm_launchers::Launcher;
use gpui::Context;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// How often the machine is looked at.
pub(crate) const LAUNCHER_REFRESH: Duration = Duration::from_secs(5);

/// The window's last observation and when it was made.
#[derive(Debug, Default)]
pub(crate) struct Launchers {
    pub(crate) observed: Observed,
    pub(crate) last_poll: Option<Instant>,
    /// A look is running, so ticks do not stack another.
    pub(crate) loading: bool,
    /// Rows being started, by (canonical host, port), and since when.
    pub(crate) starting: HashMap<(String, u16), Instant>,
    /// A terminal launcher's new tab on its way.
    pub(crate) terminal: Option<super::TerminalJob>,
}

impl HerdrWindow {
    /// Runs every window tick.
    pub(crate) fn poll_llm_launchers(&mut self, cx: &mut Context<Self>) {
        self.poll_llm_launchers_at(Instant::now(), observe, cx);
    }

    /// Looks off the UI thread when the last look is five seconds old at
    /// `now`, and notifies only when the observation changed.
    pub(crate) fn poll_llm_launchers_at(
        &mut self,
        now: Instant,
        observe: fn(&[Launcher]) -> Observed,
        cx: &mut Context<Self>,
    ) {
        if self.config.llm_launchers.is_empty() {
            return;
        }
        let due = self
            .llm_launch
            .last_poll
            .is_none_or(|at| now.saturating_duration_since(at) >= LAUNCHER_REFRESH);
        if self.llm_launch.loading || !due {
            return;
        }
        self.llm_launch.last_poll = Some(now);
        self.llm_launch.loading = true;
        let launchers = self.config.llm_launchers.clone();
        let looking = cx
            .background_executor()
            .spawn(async move { observe(&launchers) });
        cx.spawn(async move |this, cx| {
            let observed = looking.await;
            this.update(cx, |this, cx| {
                this.llm_launch.loading = false;
                if this.llm_launch.observed != observed {
                    this.llm_launch.observed = observed;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }
}
