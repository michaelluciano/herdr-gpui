//! What a click on a launcher row does beyond the plan: rows marked as
//! starting, and the start-then-open run.
use super::{
    Blocker, Health, Observed, Plan, RowInput, RowState, StartError, row_states, states, url_for,
};
use crate::HerdrWindow;
use crate::config::llm_launchers::{Kind, Launcher, canonical_host};
use crate::endpoint::LOCAL;
use crate::sidebar::blocked_message;
use crate::window::Flash;
use gpui::{Context, Window};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// How long a start is believed to be under way.
pub(crate) const START_TTL: Duration = Duration::from_secs(20 * 60);

#[cfg(test)]
thread_local! {
    /// How many times a terminal launcher reached `run_terminal_launcher`.
    pub(crate) static TERMINAL_RUNS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The effects a click has off the UI thread, injectable for tests.
#[derive(Clone, Copy)]
pub(crate) struct Effects {
    pub observe: fn(&[Launcher]) -> Observed,
    pub start: fn(&str) -> Result<(), StartError>,
}

/// Indices of llama rows whose (host, port) is in `starting` and younger
/// than [`START_TTL`] at `now`.
pub(crate) fn starting_rows(
    launchers: &[Launcher],
    starting: &HashMap<(String, u16), Instant>,
    now: Instant,
) -> HashSet<usize> {
    launchers
        .iter()
        .enumerate()
        .filter(|(_, launcher)| {
            let Some(key) = start_key(launcher) else {
                return false;
            };
            starting
                .get(&key)
                .is_some_and(|since| now.saturating_duration_since(*since) < START_TTL)
        })
        .map(|(index, _)| index)
        .collect()
}

/// The `starting` key of a llama row.
fn start_key(launcher: &Launcher) -> Option<(String, u16)> {
    match launcher.kind {
        Kind::Llama { port, .. } => Some((canonical_host(&launcher.host).to_owned(), port)),
        Kind::Terminal { .. } => None,
    }
}

/// Like `states`, but a row in `starting` that is not yet seen up counts
/// as loading.
pub(crate) fn states_with_starting(
    launchers: &[Launcher],
    observed: &Observed,
    starting: &HashSet<usize>,
) -> Vec<RowState> {
    let rows: Vec<RowInput> = launchers
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let health = observed.health.get(index).copied().flatten();
            let is_llama = matches!(row.kind, Kind::Llama { .. });
            let health = match health {
                None | Some(Health::Down) if is_llama && starting.contains(&index) => {
                    Some(Health::Loading)
                }
                other => other,
            };
            RowInput {
                host: &row.host,
                group: row.group.as_deref(),
                health,
            }
        })
        .collect();
    let foreign: Vec<(&str, u16)> = observed
        .foreign
        .iter()
        .map(|port| ("Local", *port))
        .collect();
    row_states(&rows, &foreign)
}

impl HerdrWindow {
    /// The state of every launcher row at `now`, counting rows being started.
    pub(crate) fn llm_states(&self, now: Instant) -> Vec<RowState> {
        let launchers = &self.config.llm_launchers;
        let starting = starting_rows(launchers, &self.llm_launch.starting, now);
        states_with_starting(launchers, &self.llm_launch.observed, &starting)
    }

    /// Runs a click's plan on a launcher row with the given effects.
    pub(crate) fn run_llm_plan_with(
        &mut self,
        index: usize,
        plan: Plan,
        effects: Effects,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(launcher) = self.config.llm_launchers.get(index) else {
            return;
        };
        if matches!(launcher.kind, Kind::Terminal { .. }) {
            self.run_terminal_launcher(index, window, cx);
            return;
        }
        match plan {
            Plan::Open => self.open_llm_page(index, window, cx),
            Plan::StartThenOpen => self.start_llm_then_open(index, effects, window, cx),
            Plan::Refuse { .. } | Plan::Ignore => {}
        }
    }

    fn open_llm_page(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(Launcher {
            label,
            kind: Kind::Llama { port, url, .. },
            ..
        }) = self.config.llm_launchers.get(index)
        else {
            return;
        };
        let label = label.clone();
        let url = match url_for(*port, url.as_deref()) {
            Ok(url) => url,
            Err(error) => {
                self.show_flash(Flash::warning(format!("{label}: {error}.")), cx);
                return;
            }
        };
        // The selected endpoint's live state is the window's own.
        let workspace = self
            .endpoints
            .iter()
            .position(|endpoint| endpoint.id == LOCAL)
            .and_then(|local| {
                let live = if local == self.selected_endpoint {
                    &self.live
                } else {
                    &self.endpoints[local].live
                };
                live.snapshot.as_ref()
            })
            .and_then(|snapshot| snapshot.focused_workspace_id.clone());
        let Some(workspace) = workspace else {
            self.show_flash(Flash::warning("Open a workspace on this Mac first."), cx);
            return;
        };
        self.open_workspace_page(LOCAL, &workspace, url, window, cx);
        cx.notify();
    }

    fn start_llm_then_open(
        &mut self,
        index: usize,
        effects: Effects,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let launchers = self.config.llm_launchers.clone();
        let now = Instant::now();
        if starting_rows(&launchers, &self.llm_launch.starting, now).contains(&index) {
            return;
        }
        let Some(Launcher {
            label,
            kind: Kind::Llama { start, .. },
            ..
        }) = launchers.get(index)
        else {
            return;
        };
        let (label, command) = (label.clone(), start.clone());
        let Some(key) = launchers.get(index).and_then(start_key) else {
            return;
        };
        self.llm_launch.starting.insert(key.clone(), now);
        cx.notify();
        let job = cx.background_executor().spawn(async move {
            let observed = (effects.observe)(&launchers);
            match states(&launchers, &observed).get(index) {
                Some(RowState::Running) => return Outcome::Running,
                Some(RowState::Starting) => return Outcome::Loading,
                Some(RowState::Blocked { by }) => return Outcome::Blocked(*by),
                Some(RowState::Stopped) | None => {}
            }
            match (effects.start)(&command) {
                Ok(()) => Outcome::Started,
                Err(error) => Outcome::Failed(error),
            }
        });
        cx.spawn_in(window, async move |this, cx| {
            let outcome = job.await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.llm_launch.starting.remove(&key);
                this.finish_llm_start(index, &label, outcome, window, cx);
            });
        })
        .detach();
    }

    fn finish_llm_start(
        &mut self,
        index: usize,
        label: &str,
        outcome: Outcome,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let message = match outcome {
            Outcome::Running => None,
            Outcome::Started => {
                self.llm_launch.last_poll = None;
                None
            }
            Outcome::Blocked(by) => Some(blocked_message(by, &self.config.llm_launchers)),
            Outcome::Loading => Some(format!("{label} is still loading.")),
            Outcome::Failed(StartError::Failed(Some(code))) => {
                Some(format!("{label} did not start (exit {code})."))
            }
            Outcome::Failed(StartError::Failed(None)) => {
                Some(format!("{label} did not start (killed)."))
            }
            Outcome::Failed(StartError::Spawn(error)) => {
                Some(format!("{label} did not start: {error}."))
            }
        };
        match message {
            Some(message) => self.show_flash(Flash::warning(message), cx),
            None => self.open_llm_page(index, window, cx),
        }
        cx.notify();
    }
}

/// How the background start run ended.
enum Outcome {
    Running,
    Loading,
    Blocked(Blocker),
    Started,
    Failed(StartError),
}
