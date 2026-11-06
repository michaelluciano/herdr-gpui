//! The sidebar's GitHub account chip: the active `gh` account on github.com, one
//! click from the next logged-in one. Amber when the focused workspace's repository
//! belongs to someone else. It never switches by itself; only a click does.
mod accounts;

use crate::{HerdrWindow, avatars::Checkout};
use accounts::{ChipTone, GhAccount, chip_view, next_account, parse_gh_accounts};
use gpui::{ClickEvent, Context, FontWeight, Global, Role, div, prelude::*, px, rgb};
use std::time::{Duration, Instant};

/// The chip row's height, above the device footer; the split-drag math adds it.
pub(crate) const GH_CHIP_HEIGHT: f32 = 24.;

/// How long a status read stays fresh before the next tick repeats it.
const REFRESH: Duration = Duration::from_secs(60);

/// What the last read found. One per app: every window shows the same `gh` login.
#[derive(Default)]
pub(super) struct GhChipState {
    accounts: Vec<GhAccount>,
    /// The focused workspace's repository owner, when it is a local checkout.
    owner: Option<String>,
    /// The local directory the owner was read for; a change triggers a read.
    cwd: Option<String>,
    last: Option<Instant>,
    busy: bool,
}

impl Global for GhChipState {}

#[cfg(test)]
impl GhChipState {
    /// Two logged-in accounts, `active` first, as a finished read would leave them.
    pub(super) fn fixture(active: &str, owner: Option<&str>) -> Self {
        let account = |name: &str, active| GhAccount {
            name: name.into(),
            active,
        };
        Self {
            accounts: vec![account(active, true), account("other", false)],
            owner: owner.map(str::to_owned),
            cwd: None,
            last: Some(Instant::now()),
            busy: false,
        }
    }
}

/// `gh`'s combined output, since some versions print the status to stderr.
/// Empty when `gh` is missing or could not start.
fn run_gh(args: &[&str]) -> String {
    let output = herdr_client::new_command("gh")
        .args(args)
        .env("GH_PROMPT_DISABLED", "1")
        .stdin(std::process::Stdio::null())
        .output();
    match output {
        Ok(output) => {
            let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
            text.push('\n');
            text.push_str(&String::from_utf8_lossy(&output.stderr));
            text
        }
        Err(_) => String::new(),
    }
}

impl HerdrWindow {
    /// Called every tick: starts a read at launch, when the focused workspace
    /// changes, and once a minute. Does nothing while the chip is not shown.
    pub(crate) fn poll_gh_chip(&mut self, cx: &mut Context<Self>) {
        if !self.device_footer_shown() {
            return;
        }
        let cwd = self.gh_chip_cwd();
        let state = cx.default_global::<GhChipState>();
        let due = state.last.is_none_or(|last| last.elapsed() >= REFRESH) || state.cwd != cwd;
        if !state.busy && due {
            self.refresh_gh_chip(None, cwd, cx);
        }
    }

    /// The focused workspace's directory when this machine's Git can read it.
    fn gh_chip_cwd(&self) -> Option<String> {
        if self.selected_endpoint != 0 {
            return None;
        }
        let target = &self.endpoints.first()?.connection.target;
        let snapshot = self.live.snapshot.as_ref()?;
        let id = snapshot.focused_workspace_id.as_ref()?;
        let workspace = snapshot
            .workspaces
            .iter()
            .find(|workspace| &workspace.workspace_id == id)?;
        match Checkout::of(true, target, workspace)? {
            Checkout::Local(cwd) => Some(cwd.to_owned()),
            Checkout::Ssh { .. } => None,
        }
    }

    /// Optionally switches to `switch_to`, then reads the accounts and the owner
    /// of `cwd` off the UI thread and stores them.
    fn refresh_gh_chip(
        &mut self,
        switch_to: Option<String>,
        cwd: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let state = cx.default_global::<GhChipState>();
        state.busy = true;
        state.cwd = cwd.clone();
        let work = cx.background_executor().spawn(async move {
            if let Some(name) = switch_to {
                run_gh(&["auth", "switch", "-h", "github.com", "-u", &name]);
            }
            let accounts = parse_gh_accounts(&run_gh(&["auth", "status"]));
            let owner = cwd.as_deref().and_then(|cwd| {
                crate::avatars::repo_owner(Checkout::Local(cwd), &Default::default())
                    .ok()
                    .flatten()
            });
            (accounts, owner)
        });
        cx.spawn(async move |this, cx| {
            let (accounts, owner) = work.await;
            let _ = this.update(cx, |_, cx| {
                let state = cx.default_global::<GhChipState>();
                state.accounts = accounts;
                state.owner = owner;
                state.last = Some(Instant::now());
                state.busy = false;
                cx.notify();
            });
        })
        .detach();
    }

    /// The one-row chip above the device footer.
    pub(super) fn render_gh_chip(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.default_global::<GhChipState>();
        let view = chip_view(&state.accounts, state.owner.as_deref());
        let tokens = self.theme.tokens();
        let color = match view.tone {
            ChipTone::Match | ChipTone::Unknown => tokens.dim,
            ChipTone::Missing => tokens.faint,
            ChipTone::Mismatch => tokens.working,
        };
        let hint = match view.tone {
            ChipTone::Mismatch => "GitHub account differs from this repository's owner",
            _ => "GitHub account",
        };
        div()
            .id("gh-chip")
            .role(Role::Button)
            .aria_label(format!("{hint}: {}", view.label))
            .debug_selector(|| "gh-chip".into())
            .h(px(GH_CHIP_HEIGHT))
            .flex_none()
            .flex()
            .items_center()
            .px(px(14.))
            .text_size(px(11.))
            .text_color(rgb(color))
            .when(view.tone == ChipTone::Mismatch, |chip| {
                chip.font_weight(FontWeight::SEMIBOLD)
            })
            .cursor_pointer()
            .child(div().min_w_0().truncate().child(view.label))
            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                let state = cx.default_global::<GhChipState>();
                let next = next_account(&state.accounts).map(str::to_owned);
                if let Some(name) = next.filter(|_| !state.busy) {
                    let cwd = state.cwd.clone();
                    this.refresh_gh_chip(Some(name), cwd, cx);
                }
            }))
    }
}
