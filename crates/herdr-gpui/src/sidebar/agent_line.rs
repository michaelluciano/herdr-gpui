//! The second line of a default agent row: the agent icon, a `|`, the tab's
//! number `[n]`, then the status or activity text. The name owns the first
//! line, so nothing beside it can cut it short. The icon, the `|` and the tag
//! keep their width; the status text clips first.

use super::{
    glyph_width, label_text, line_height,
    row::{agent_icon_size, agent_mark},
};
use crate::{config::FontConfig, icons::AgentIcon};
use gpui::{prelude::*, *};
use unicode_width::UnicodeWidthStr;

/// What the second line shows after the icon.
#[derive(Clone, Copy)]
pub(super) struct AgentLine<'a> {
    /// The tab's 1-based number; none omits it and the `|` before it.
    pub(super) tab: Option<usize>,
    pub(super) status: Option<&'a str>,
    pub(super) status_color: u32,
}

/// The tab tag, as the tab bar numbers it.
pub(super) fn tab_tag(number: usize) -> String {
    format!("[{number}]")
}

/// The widths of the separator, the tab tag and the status text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Widths {
    pub(super) sep: f32,
    pub(super) tab: f32,
    pub(super) status: f32,
}

/// Splits `room` after the icon, one glyph apart. The separator and tag are
/// never cut; the status takes what is left, up to its own length.
pub(super) fn widths(
    room: f32,
    glyph: f32,
    tab_chars: Option<usize>,
    status_chars: usize,
) -> Widths {
    let (sep, tab) = tab_chars.map_or((0., 0.), |chars| (glyph, chars as f32 * glyph));
    let before = if tab_chars.is_some() { 2. * glyph } else { 0. };
    let status = (status_chars as f32 * glyph).min((room - sep - tab - before).max(0.));
    Widths { sep, tab, status }
}

pub(super) fn line(
    key: &str,
    icon: AgentIcon,
    parts: AgentLine<'_>,
    limit: f32,
    (font, muted): (&FontConfig, u32),
) -> Div {
    let (size, reserve) = agent_icon_size(font);
    let glyph = glyph_width(font);
    let tag = parts.tab.map(tab_tag);
    let status_chars = parts.status.map_or(0, |text| text.width());
    let room = (limit - reserve).max(0.);
    let width = widths(room, glyph, tag.as_deref().map(str::len), status_chars);
    div()
        .relative()
        .w(px(limit))
        .h(px(line_height(font)))
        .overflow_hidden()
        .child(agent_mark(key, icon, size, muted, font))
        .child(
            div()
                .absolute()
                .top_0()
                .left(px(reserve.min(limit)))
                .h(px(line_height(font)))
                .flex()
                .items_center()
                .gap(px(glyph))
                .text_color(rgb(muted))
                .children(
                    tag.map(|tag| {
                        [
                            div()
                                .debug_selector(|| format!("agent-sep-{key}"))
                                .w(px(width.sep))
                                .flex_none()
                                .child(label_text("|"))
                                .into_any_element(),
                            div()
                                .debug_selector(|| format!("agent-tab-{key}"))
                                .w(px(width.tab))
                                .flex_none()
                                .child(label_text(&tag))
                                .into_any_element(),
                        ]
                    })
                    .into_iter()
                    .flatten(),
                )
                .children(parts.status.filter(|_| width.status > 0.).map(|text| {
                    div()
                        .debug_selector(|| format!("status-{key}"))
                        .w(px(width.status))
                        .flex_none()
                        .truncate()
                        .text_color(rgb(parts.status_color))
                        .child(label_text(text))
                })),
        )
}
