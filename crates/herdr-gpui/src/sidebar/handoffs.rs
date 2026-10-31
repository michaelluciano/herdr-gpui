//! The Handoffs section below the workspaces and agents lists.

use super::{label_text, layout::SidebarLook, line_height, render::header};
use crate::{
    HerdrWindow,
    handoffs::{Handoff, target_for},
    window::chrome_focus::Region,
};
use gpui::{prelude::*, *};

/// The share of the sidebar's height the section may take.
const MAX_SHARE: f32 = 0.30;
/// How many characters of `created` a row shows: the date.
const DATE_CHARS: usize = 10;

/// A row's second line: the project, then the day it was written.
fn detail(handoff: &Handoff) -> String {
    let date: String = handoff.created.chars().take(DATE_CHARS).collect();
    if date.is_empty() {
        handoff.project.clone()
    } else {
        format!("{} \u{b7} {date}", handoff.project)
    }
}

impl HerdrWindow {
    /// The section, or `None` when there is nothing to list or the layout
    /// lists devices. Its rows scroll inside a cap so the lists above keep room.
    pub(super) fn handoffs_section(
        &self,
        devices: bool,
        look: SidebarLook,
        cx: &mut Context<Self>,
    ) -> Option<Div> {
        let list = &self.handoffs.list;
        if devices || list.is_empty() {
            return None;
        }
        let font = &self.config.sidebar;
        let theme = &self.theme;
        let rows = list.iter().enumerate().map(|(index, handoff)| {
            let target = target_for(handoff);
            self.chrome_stop(
                Region::Sidebar,
                div()
                    .id(("handoff-row", index))
                    .debug_selector(|| format!("handoff-row-{index}"))
                    .role(Role::Button)
                    .aria_label(format!("Open handoff {}", handoff.id))
                    .flex_none()
                    .min_w_0()
                    .px(px(look.content_x()))
                    .py(px(look.row_padding()))
                    .cursor_pointer()
                    .hover(|row| row.bg(rgba(crate::panel_resize::RESIZE_HOVER)))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_code_view(&target, window, cx);
                    }))
                    .child(
                        div()
                            .truncate()
                            .text_color(rgb(theme.subtext()))
                            .child(label_text(&handoff.id)),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_color(rgb(theme.muted))
                            .child(label_text(&detail(handoff))),
                    ),
            )
        });
        Some(
            div()
                .debug_selector(|| "handoffs-section".into())
                .flex_none()
                .min_h_0()
                .max_h(relative(MAX_SHARE))
                .flex()
                .flex_col()
                .overflow_hidden()
                .border_t_1()
                .border_color(rgb(theme.tokens().line))
                .child(
                    header("handoffs", font, theme, look)
                        .justify_between()
                        .child(label_text(&list.len().to_string())),
                )
                .child(
                    div()
                        .id("handoffs-scroll")
                        .debug_selector(|| "handoffs-scroll".into())
                        .flex_initial()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .overflow_y_scroll()
                        .text_size(px(font.size))
                        .line_height(px(line_height(font)))
                        .children(rows),
                ),
        )
    }
}
