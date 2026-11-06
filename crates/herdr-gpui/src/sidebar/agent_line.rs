//! The second line of a default agent row: the agent icon, a `|`, the tab's
//! number `[n]`, then the status or activity text. The name owns the first
//! line, so nothing beside it can cut it short. The icon, the `|` and the tag
//! keep their width; the status text clips first.

use super::{
    glyph_width, label_text, line_height,
    row::{agent_icon_size, agent_mark},
};
use crate::{
    agent_activity::Glyph,
    config::{FontConfig, Theme},
    icons::AgentIcon,
};
use gpui::{prelude::*, *};
use unicode_width::UnicodeWidthStr;

/// What the second line shows after the icon.
#[derive(Clone, Copy)]
pub(super) struct AgentLine<'a> {
    /// The tab's 1-based number; none omits it and the `|` before it.
    pub(super) tab: Option<usize>,
    /// A mark standing in for the state's word, with that word for its tooltip.
    pub(super) glyph: Option<(Glyph, &'a str)>,
    pub(super) status: Option<&'a str>,
    pub(super) status_color: u32,
    /// The model letter beside the icon.
    pub(super) letter: Option<char>,
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
    pub(super) icon: f32,
    pub(super) status: f32,
}

/// Splits `room` after the agent icon, one glyph apart. The separator, the
/// tag and the status mark (`icon` wide, 0 for none) are never cut; the
/// status text takes what is left, up to its own length.
pub(super) fn widths(
    room: f32,
    glyph: f32,
    tab_chars: Option<usize>,
    icon: f32,
    status_chars: usize,
) -> Widths {
    let (sep, tab) = tab_chars.map_or((0., 0.), |chars| (glyph, chars as f32 * glyph));
    let fixed = [sep, tab, icon].iter().filter(|width| **width > 0.).count();
    let gaps = match fixed {
        0 => 0,
        n => n - 1 + usize::from(status_chars > 0),
    };
    let status =
        (status_chars as f32 * glyph).min((room - sep - tab - icon - gaps as f32 * glyph).max(0.));
    Widths {
        sep,
        tab,
        icon,
        status,
    }
}

pub(super) fn line(
    key: &str,
    icon: AgentIcon,
    parts: AgentLine<'_>,
    limit: f32,
    (font, theme, muted): (&FontConfig, &Theme, u32),
) -> Div {
    let (size, reserve) = agent_icon_size(font, parts.letter);
    let glyph = glyph_width(font);
    let tag = parts.tab.map(tab_tag);
    let status_chars = parts.status.map_or(0, |text| text.width());
    let room = (limit - reserve).max(0.);
    let mark = parts.glyph.map(|(glyph, words)| (glyph, words, size));
    let width = widths(
        room,
        glyph,
        tag.as_deref().map(str::len),
        mark.map_or(0., |(_, _, size)| size),
        status_chars,
    );
    div()
        .relative()
        .w(px(limit))
        .h(px(line_height(font)))
        .overflow_hidden()
        .child(agent_mark(
            key,
            icon,
            size,
            muted,
            font,
            parts.letter.map(|letter| (letter, theme.muted)),
        ))
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
                .children(mark.map(|(glyph, words, size)| {
                    status_mark(key, glyph, words, size, parts.status_color, theme)
                }))
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

/// The status mark: its own colour and size, since an svg paints with
/// neither it inherits, and a tooltip with the words it stands for.
fn status_mark(
    key: &str,
    glyph: Glyph,
    words: &str,
    size: f32,
    color: u32,
    theme: &Theme,
) -> impl IntoElement {
    let text = SharedString::from(words.to_owned());
    let (foreground, surface) = (theme.foreground, theme.surface);
    div()
        .id(SharedString::from(format!("status-glyph-{key}")))
        .size(px(size))
        .flex_none()
        .child(
            svg()
                .debug_selector(|| format!("status-icon-{key}"))
                .path(glyph.path())
                .w(px(size))
                .h(px(size))
                .flex_none()
                .text_color(rgb(color)),
        )
        .tooltip(move |_, cx| {
            cx.new(|_| crate::usage::Hint {
                text: text.clone(),
                foreground,
                surface,
            })
            .into()
        })
}
