//! The LLM launcher rows under each host header in the spaces list.

use super::{label_text, layout::SidebarLook};
use crate::{
    HerdrWindow,
    config::{Theme, llm_launchers::Launcher},
    endpoint::Endpoint,
    llm_launch::{Blocker, Plan, RowState},
    window::{Flash, chrome_focus::Region},
};
use gpui::{
    Context, Div, InteractiveElement, ParentElement, Role, SharedString, Stateful,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px, rgb, rgba,
};

/// How a row looks and whether a click does anything.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RowLook {
    pub muted: bool,
    pub clickable: bool,
    pub suffix: Option<&'static str>,
}

pub(super) fn row_look(state: RowState) -> RowLook {
    let (muted, clickable, suffix) = match state {
        RowState::Stopped => (false, true, None),
        RowState::Starting => (false, false, Some("starting")),
        RowState::Running => (false, true, Some("running")),
        RowState::Blocked { .. } => (true, false, None),
    };
    RowLook {
        muted,
        clickable,
        suffix,
    }
}

/// Why a blocked row cannot start, for the flash a click on it shows.
pub(crate) fn blocked_message(by: Blocker, launchers: &[Launcher]) -> String {
    match by {
        Blocker::Row(index) => match launchers.get(index) {
            Some(launcher) => format!("{} is running. Stop it first.", launcher.label),
            None => "Another model is running. Stop it first.".into(),
        },
        Blocker::Foreign { port } => {
            format!("Another model is running on port {port}. Stop it first.")
        }
    }
}

pub(super) fn label_color(look: RowLook, theme: &Theme) -> u32 {
    if look.muted {
        theme.muted
    } else {
        theme.subtext()
    }
}

#[cfg(test)]
thread_local! {
    /// How many times a click reached `run_llm_plan`.
    pub(super) static PLAN_RUNS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

impl HerdrWindow {
    /// One row per launcher of `endpoint`, each exactly one child of `spaces`;
    /// returns the list and how many rows it gained. `indent` is where a
    /// workspace row's text starts, which a launcher label lines up with.
    pub(super) fn append_llm_rows(
        &self,
        mut spaces: Stateful<Div>,
        endpoint: &Endpoint,
        indent: f32,
        look: SidebarLook,
        cx: &mut Context<Self>,
    ) -> (Stateful<Div>, usize) {
        let launchers = &self.config.llm_launchers;
        let rows = crate::llm_launch::rows_for_host(launchers, &endpoint.label, &endpoint.id);
        let states = self.llm_states(std::time::Instant::now());
        let theme = &self.theme;
        let count = rows.len();
        for (index, launcher) in rows {
            let state = states.get(index).copied().unwrap_or(RowState::Stopped);
            let row_look = row_look(state);
            let id = endpoint.id.as_str();
            let blocked = matches!(state, RowState::Blocked { .. });
            let inner = div()
                .flex()
                .items_center()
                .gap(px(look.content_x()))
                .when(blocked, |inner| {
                    inner.debug_selector({
                        let id = id.to_owned();
                        move || format!("llm-row-blocked-{id}-{index}")
                    })
                })
                .child(
                    div()
                        .debug_selector({
                            let id = id.to_owned();
                            move || format!("llm-row-label-{id}-{index}")
                        })
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_color(rgb(label_color(row_look, theme)))
                        .child(label_text(&launcher.label)),
                )
                .children(row_look.suffix.map(|suffix| {
                    div()
                        .debug_selector({
                            let id = id.to_owned();
                            move || format!("llm-row-suffix-{id}-{index}")
                        })
                        .flex_none()
                        .text_color(rgb(theme.muted))
                        .child(label_text(suffix))
                }));
            let row = self.chrome_stop(
                Region::Sidebar,
                div()
                    .id(SharedString::from(format!("llm-row-{id}-{index}")))
                    .debug_selector({
                        let id = id.to_owned();
                        move || format!("llm-row-{id}-{index}")
                    })
                    .role(Role::Button)
                    .aria_label(format!("Open {}", launcher.label))
                    .flex_none()
                    .min_w_0()
                    .pl(px(indent))
                    .pr(px(look.content_x()))
                    .py(px(look.row_padding()))
                    .when(row_look.clickable, |row| {
                        row.cursor_pointer()
                            .hover(|row| row.bg(rgba(crate::panel_resize::RESIZE_HOVER)))
                    })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.click_llm_launcher(index, window, cx);
                    }))
                    .child(inner),
            );
            spaces = spaces.child(row);
        }
        (spaces, count)
    }

    /// Flashes why a blocked or loading row does nothing; otherwise hands the
    /// row's plan to `run_llm_plan`.
    pub(crate) fn click_llm_launcher(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let launchers = &self.config.llm_launchers;
        let Some(launcher) = launchers.get(index) else {
            return;
        };
        let states = self.llm_states(std::time::Instant::now());
        let Some(state) = states.get(index).copied() else {
            return;
        };
        match crate::llm_launch::plan(state) {
            Plan::Refuse { by } => {
                let message = blocked_message(by, launchers);
                self.show_flash(Flash::warning(message), cx);
            }
            Plan::Ignore => {
                let message = format!("{} is still loading.", launcher.label);
                self.show_flash(Flash::warning(message), cx);
            }
            plan @ (Plan::Open | Plan::StartThenOpen) => {
                self.run_llm_plan(index, plan, window, cx);
            }
        }
    }

    pub(super) fn run_llm_plan(
        &mut self,
        index: usize,
        plan: Plan,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Tests count the click; the probes and `sh` behind it stay unrun.
        #[cfg(test)]
        {
            let _ = (index, plan, window, cx);
            PLAN_RUNS.with(|runs| runs.set(runs.get() + 1));
        }
        #[cfg(not(test))]
        {
            let effects = crate::llm_launch::Effects {
                observe: crate::llm_launch::observe,
                start: crate::llm_launch::start,
            };
            self.run_llm_plan_with(index, plan, effects, window, cx);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
