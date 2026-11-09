//! What an agent row shows of its Claude transcript: the model's letter beside
//! the agent icon and the context size at the row's far right. Pure data and
//! two small elements; each layout decides where they sit in its own line.

use super::{glyph_width, label_text, line_height};
use crate::{
    agent_activity::{ContextBand, Stats, context_band, format_context, model_letter},
    config::{FontConfig, Theme},
};
use gpui::{prelude::*, *};

/// The gap between the agent icon and its letter.
const LETTER_GAP: f32 = 2.;

/// One agent's letter and context, resolved once per row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StatsMark {
    /// H, S, O or F; none for a model this app does not name.
    pub(super) letter: Option<char>,
    /// The context size as `115k`.
    pub(super) context: String,
    /// The context text's colour: muted until the compact limit nears.
    pub(super) color: u32,
}

impl StatsMark {
    pub(super) fn new(stats: &Stats, limit: Option<u64>, theme: &Theme) -> Self {
        Self {
            letter: model_letter(&stats.model),
            context: format_context(stats.context),
            color: context_color(context_band(stats.context, limit), theme),
        }
    }

    /// The width the context column keeps.
    pub(super) fn context_width(&self, font: &FontConfig) -> f32 {
        (self.context.chars().count() as f32 * glyph_width(font)).ceil()
    }

    /// The context column: it keeps its width, on one line.
    pub(super) fn context_element(&self, font: &FontConfig) -> Div {
        div()
            .debug_selector(|| "agent-context".to_owned())
            .w(px(self.context_width(font)))
            .h(px(line_height(font)))
            .flex_none()
            .flex()
            .items_center()
            .justify_end()
            .overflow_hidden()
            .whitespace_nowrap()
            .text_color(rgb(self.color))
            .child(label_text(&self.context))
    }
}

/// The colour of the context text in `band`: the theme's muted ink, then its
/// working colour, then its blocked colour.
pub(super) fn context_color(band: ContextBand, theme: &Theme) -> u32 {
    match band {
        ContextBand::Normal => theme.muted,
        ContextBand::Warm => theme.ink(theme.tokens().working),
        ContextBand::Alert => theme.ink(theme.tokens().blocked),
    }
}

/// Width of a letter column.
pub(super) fn letter_width(font: &FontConfig) -> f32 {
    glyph_width(font).ceil()
}

/// Where the letter starts, after an icon `size` wide.
pub(super) fn letter_x(size: f32) -> f32 {
    size + LETTER_GAP
}

/// Room the letter takes beyond the icon's own, gap included.
pub(super) fn letter_reserve(font: &FontConfig, letter: Option<char>) -> f32 {
    letter.map_or(0., |_| LETTER_GAP + letter_width(font))
}

/// The model letter, muted, one line tall.
pub(super) fn letter_element(letter: char, muted: u32, font: &FontConfig) -> Div {
    div()
        .debug_selector(|| "agent-model-letter".to_owned())
        .w(px(letter_width(font)))
        .h(px(line_height(font)))
        .flex_none()
        .flex()
        .items_center()
        .text_color(rgb(muted))
        .child(label_text(&letter.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{StatsMark, context_color};
    use crate::{
        agent_activity::{ContextBand, Stats},
        config::Theme,
    };

    #[test]
    fn the_context_colour_follows_the_band() {
        let theme = Theme::default();
        let normal = context_color(ContextBand::Normal, &theme);
        let warm = context_color(ContextBand::Warm, &theme);
        let alert = context_color(ContextBand::Alert, &theme);
        assert_eq!(normal, theme.muted);
        assert_eq!(warm, theme.ink(theme.tokens().working));
        assert_eq!(alert, theme.ink(theme.tokens().blocked));
        assert!(normal != warm && warm != alert && normal != alert);
    }

    #[test]
    fn a_mark_names_the_model_and_the_band_from_the_limit() {
        let theme = Theme::default();
        let stats = Stats {
            model: "claude-opus-5".into(),
            context: 117_300,
        };
        let mark = StatsMark::new(&stats, Some(250_000), &theme);
        assert_eq!((mark.letter, mark.context.as_str()), (Some('O'), "115k"));
        assert_eq!(mark.color, theme.muted);
        let near = StatsMark::new(&stats, Some(125_000), &theme);
        assert_eq!(near.color, context_color(ContextBand::Alert, &theme));
        let odd = Stats {
            model: "other".into(),
            context: 100,
        };
        assert_eq!(StatsMark::new(&odd, None, &theme).letter, None);
    }
}
