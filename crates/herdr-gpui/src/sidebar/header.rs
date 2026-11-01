//! The bar that titles a sidebar section, shared by every section header.

use super::{label_text, layout::SidebarLook, line_height};
use crate::config::{FontConfig, Theme};
use gpui::{prelude::*, *};

pub(super) fn header(
    label: &'static str,
    font: &FontConfig,
    theme: &Theme,
    look: SidebarLook,
) -> Div {
    header_frame(label, font, theme, look).child(
        div()
            .debug_selector(|| format!("header-label-{label}"))
            .child(label_text(&look.header_label(label))),
    )
}

/// A section header's bar without its label, for a header whose title holds
/// controls of its own.
pub(super) fn header_frame(
    label: &'static str,
    font: &FontConfig,
    theme: &Theme,
    look: SidebarLook,
) -> Div {
    div()
        .debug_selector(|| format!("header-{label}"))
        .flex_none()
        .h(px(line_height(font) + 2. * look.density.header_padding()))
        .px(px(look.content_x()))
        .flex()
        .items_center()
        .text_size(px(font.size))
        .text_color(rgb(theme.muted))
}
