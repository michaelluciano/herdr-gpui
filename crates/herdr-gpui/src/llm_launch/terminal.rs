//! A terminal launcher row's click: a new terminal tab on the row's host
//! with its `run` line typed into the tab's shell, following the closed
//! agents' reopen (`tab.create` through the script slot, then one typed line).
use crate::config::llm_launchers::{Kind, canonical_host};
use crate::{HerdrWindow, NavigationTarget, window::Flash};
use gpui::{Context, Window};
use herdr_client::Method;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

/// How long a terminal launcher may wait for its host to come up and answer.
pub(crate) const DEADLINE: Duration = Duration::from_secs(15);

/// A terminal launcher's tab on its way.
#[derive(Debug)]
pub(crate) struct TerminalJob {
    pub(crate) endpoint: String,
    pub(crate) label: String,
    pub(crate) line: String,
    pub(crate) deadline: Instant,
    /// The boot the first snapshot named.
    pub(crate) boot: Option<String>,
    /// The `tab.create` request id.
    pub(crate) sent: Option<String>,
}

/// What a click does before touching any host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Gate {
    /// A job is already on its way: nothing.
    Pending,
    /// Another feature holds the script slot: say so.
    Busy,
    Go,
}

/// The first endpoint `host` names, by label or id, with the launcher rows'
/// own matching rule.
pub(crate) fn resolve_endpoint(endpoints: &[(&str, &str)], host: &str) -> Option<usize> {
    let host = canonical_host(host);
    endpoints
        .iter()
        .position(|(id, label)| host == canonical_host(label) || host == canonical_host(id))
}

/// The focused workspace when it is listed, else the first.
pub(crate) fn pick_workspace(focused: Option<&str>, workspaces: &[&str]) -> Option<String> {
    focused
        .filter(|focused| workspaces.contains(focused))
        .or_else(|| workspaces.first().copied())
        .map(str::to_owned)
}

/// The `tab.create` parameters.
pub(crate) fn create_params(workspace: &str) -> Value {
    json!({"workspace_id": workspace, "focus": true})
}

/// Pending wins over busy, since a pending job is what makes the slot busy.
pub(crate) fn click_gate(busy: bool, pending: bool) -> Gate {
    if pending {
        Gate::Pending
    } else if busy {
        Gate::Busy
    } else {
        Gate::Go
    }
}

impl HerdrWindow {
    /// Runs a terminal launcher row.
    pub(crate) fn run_terminal_launcher(
        &mut self,
        index: usize,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        #[cfg(test)]
        super::click::TERMINAL_RUNS.with(|runs| runs.set(runs.get() + 1));
        let Some(launcher) = self.config.llm_launchers.get(index) else {
            return;
        };
        let Kind::Terminal { run } = &launcher.kind else {
            return;
        };
        let (label, host, line) = (launcher.label.clone(), launcher.host.clone(), run.clone());
        let names: Vec<(&str, &str)> = self
            .endpoints
            .iter()
            .map(|endpoint| (endpoint.id.as_str(), endpoint.label.as_str()))
            .collect();
        let Some(target) = resolve_endpoint(&names, &host) else {
            self.show_flash(
                Flash::warning(format!("{label}: host {host} is not set up.")),
                cx,
            );
            return;
        };
        match click_gate(self.script_slot_busy(), self.llm_launch.terminal.is_some()) {
            Gate::Pending => return,
            Gate::Busy => {
                self.show_flash(Flash::warning("Another tab is still opening"), cx);
                return;
            }
            Gate::Go => {}
        }
        let endpoint = self.endpoints[target].id.clone();
        if self.endpoints[self.selected_endpoint].id != endpoint
            && (self.menu.page.is_some() || !self.select_endpoint(&endpoint, cx))
        {
            self.show_flash(Flash::warning(format!("Cannot switch to {host}")), cx);
            return;
        }
        self.llm_launch.terminal = Some(TerminalJob {
            endpoint,
            label,
            line,
            deadline: Instant::now() + DEADLINE,
            boot: None,
            sent: None,
        });
        self.poll_llm_terminal(cx);
    }

    /// Advances a terminal launcher's tab every tick: sends its `tab.create`
    /// once its host is the shown one, then types the row's line into the
    /// new pane.
    pub(crate) fn poll_llm_terminal(&mut self, cx: &mut Context<Self>) {
        let Some(job) = &self.llm_launch.terminal else {
            return;
        };
        let (endpoint, seen, sent) = (job.endpoint.clone(), job.boot.clone(), job.sent.clone());
        if Instant::now() >= job.deadline {
            return self.fail_llm_terminal("the host did not answer in time", cx);
        }
        if self.endpoints[self.selected_endpoint].id != endpoint {
            return;
        }
        let Some(snapshot) = self
            .live
            .snapshot
            .as_deref()
            .filter(|_| self.live.status.is_connected())
        else {
            return;
        };
        let current = snapshot.boot_id.clone();
        let boot = match &seen {
            Some(boot) if *boot != current => {
                return self.fail_llm_terminal(crate::Error::StaleConnection, cx);
            }
            Some(boot) => boot.clone(),
            None => {
                let boot = current;
                if let Some(job) = &mut self.llm_launch.terminal {
                    job.boot = Some(boot.clone());
                }
                boot
            }
        };
        let Some(request) = sent else {
            let ids: Vec<&str> = snapshot
                .workspaces
                .iter()
                .map(|workspace| workspace.workspace_id.as_str())
                .collect();
            let Some(workspace) = pick_workspace(snapshot.focused_workspace_id.as_deref(), &ids)
            else {
                return self.fail_llm_terminal("no workspace on that host", cx);
            };
            if self.script_job_busy() {
                return;
            }
            let sent = self.endpoints[self.selected_endpoint]
                .connection
                .request_script(&boot, Method::TabCreate, create_params(&workspace));
            match sent {
                Ok(request) => {
                    if let Some(job) = &mut self.llm_launch.terminal {
                        job.sent = Some(request);
                    }
                }
                Err(error) => self.fail_llm_terminal(error, cx),
            }
            return;
        };
        let Some(result) = self.take_script_response(&request) else {
            return;
        };
        let Some(job) = self.llm_launch.terminal.take() else {
            return;
        };
        let typed = result
            .map_err(|error| error.to_string())
            .and_then(|response| {
                crate::worktree_scripts::created_tab(&response).map_err(|error| error.to_string())
            })
            .and_then(|(tab, pane)| {
                self.type_script_line(&boot, &pane, &job.line)
                    .map_err(|error| error.to_string())?;
                Ok(tab)
            });
        match typed {
            Ok(tab) => {
                self.navigate_endpoint(&job.endpoint, NavigationTarget::Tab(&tab), cx);
                cx.notify();
            }
            Err(error) => {
                self.show_flash(
                    Flash::warning(format!("{} did not open: {error}", job.label)),
                    cx,
                );
            }
        }
    }

    fn fail_llm_terminal(&mut self, why: impl std::fmt::Display, cx: &mut Context<Self>) {
        let Some(job) = self.llm_launch.terminal.take() else {
            return;
        };
        self.show_flash(
            Flash::warning(format!("{} did not open: {why}", job.label)),
            cx,
        );
    }
}
