//! The one status dot, in the default `dots` set: the sidebar, tabs, worktree
//! group summaries and the Overview all draw an agent's state with it. A
//! working agent pulses in amber, an idle one is a hollow gray ring, a done
//! one a filled green disc, a blocked one a filled red disc. Colors are the
//! theme's named tokens.

use crate::{config::theme_tokens::Tokens, sidebar::STATUS_DOT_UNKNOWN};
use gpui::{prelude::*, *};
use herdr_client::protocol::AgentStatus;
use std::{f32::consts::TAU, time::Duration};

const PULSE_MS: u64 = 1400;
/// The dimmest a pulsing dot gets; it never fades out of sight.
const PULSE_LOW: f32 = 0.35;

/// What a status looks like as a dot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Dot {
    pub(crate) color: u32,
    pub(crate) filled: bool,
    pub(crate) pulses: bool,
    /// Unknown has nothing to report, so it is a speck rather than a dot.
    pub(crate) small: bool,
    bold: bool,
}

/// `status`'s dot in the theme's token colors.
pub(crate) fn spec(status: AgentStatus, tokens: &Tokens) -> Dot {
    Dot::new(
        status,
        match status {
            AgentStatus::Working => tokens.working,
            AgentStatus::Idle => tokens.idle,
            AgentStatus::Done => tokens.done,
            AgentStatus::Blocked => tokens.blocked,
            AgentStatus::Unknown => tokens.faint,
        },
    )
}

/// The opacity of a pulsing dot `delta` of the way round its loop. It starts
/// full, so reduced motion, which holds an animation on its first frame,
/// leaves a solid dot.
pub(crate) fn pulse_opacity(delta: f32) -> f32 {
    PULSE_LOW + (1. - PULSE_LOW) * (1. + (delta * TAU).cos()) / 2.
}

impl Dot {
    /// `status`'s shape in `color`, for callers that resolve color themselves
    /// (configured tokens, a contrast-adjusted theme).
    pub(crate) fn new(status: AgentStatus, color: u32) -> Self {
        Self {
            color,
            filled: status != AgentStatus::Idle,
            pulses: status == AgentStatus::Working,
            small: status == AgentStatus::Unknown,
            bold: false,
        }
    }

    /// A configured weight thickens the ring and keeps the full size.
    pub(crate) fn bold(self, bold: bool) -> Self {
        Self { bold, ..self }
    }

    /// The name this dot's shape carries, which tests look it up by.
    pub(crate) fn selector(self) -> &'static str {
        match (self.pulses, self.filled) {
            (true, _) => "status-dot-pulse",
            (false, false) => "status-dot-hollow",
            (false, true) => "status-dot-filled",
        }
    }

    /// The dot `diameter` across. A pulse runs on the app's synced clock, so
    /// every working agent breathes in step, with repaints capped because a
    /// list can show many.
    pub(crate) fn drawn(self, diameter: f32) -> AnyElement {
        let diameter = if self.small && !self.bold {
            STATUS_DOT_UNKNOWN
        } else {
            diameter
        };
        let selector = self.selector();
        let dot = div()
            .debug_selector(move || selector.into())
            .size(px(diameter))
            .flex_none()
            .rounded_full()
            .border_1()
            .when(self.bold, |dot| dot.border_2())
            .border_color(rgb(self.color))
            .when(self.filled, |dot| dot.bg(rgb(self.color)));
        if !self.pulses {
            return dot.into_any_element();
        }
        dot.with_animation(
            "status-dot-pulse",
            Animation::new(crate::motion::duration(Duration::from_millis(PULSE_MS)))
                .repeat_synced()
                .with_max_fps(30.),
            |dot, delta| dot.opacity(pulse_opacity(delta)),
        )
        .into_any_element()
    }
}

#[cfg(test)]
mod tests;
