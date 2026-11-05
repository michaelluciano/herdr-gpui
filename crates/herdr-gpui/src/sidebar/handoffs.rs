//! The Handoffs section below the workspaces and agents lists: the newest
//! unclaimed handoff of each project, and behind a header control the
//! picked-up and superseded ones.

use super::{
    accessibility::expandable, label_text, layout::SidebarLook, line_height, render::header_frame,
};
use crate::{
    HerdrWindow,
    config::{Theme, mix},
    handoffs::{Handoff, Shelf, detail, target_for},
    window::{Flash, chrome_focus::Region},
};
use gpui::{ClipboardItem, prelude::*, *};
use std::path::{Path, PathBuf};

/// The share of the sidebar's height the section may take.
const MAX_SHARE: f32 = 0.30;
/// The chevron before the title while the body shows.
const CHEVRON_OPEN: &str = "\u{25be}";
/// The chevron before the title while the body is folded.
const CHEVRON_FOLDED: &str = "\u{25b8}";

/// The colours of one handoff row.
pub(super) struct RowColors {
    /// The chip fill of an active row; `None` keeps the sidebar's own.
    pub(super) bg: Option<u32>,
    /// The fill while the pointer is over an active row.
    pub(super) hover_bg: Option<u32>,
    pub(super) id_text: u32,
    pub(super) detail_text: u32,
}

/// An active row (`shelf == None`) is the theme's pair inverted: a
/// foreground-coloured chip with background-coloured text, the detail line
/// a step toward the chip. A history row keeps the dimmed look.
pub(super) fn row_colors(theme: &Theme, shelf: Option<Shelf>) -> RowColors {
    match shelf {
        None => RowColors {
            bg: Some(theme.foreground),
            hover_bg: Some(mix(theme.foreground, theme.background, 6)),
            id_text: theme.background,
            detail_text: mix(theme.background, theme.foreground, 16),
        },
        Some(_) => RowColors {
            bg: None,
            hover_bg: None,
            id_text: theme.muted,
            detail_text: theme.muted,
        },
    }
}

fn flip_folded(this: &mut HerdrWindow, cx: &mut Context<HerdrWindow>) {
    this.handoffs.collapsed = !this.handoffs.collapsed;
    cx.notify();
}

fn flip_history(this: &mut HerdrWindow, cx: &mut Context<HerdrWindow>) {
    this.handoffs.history_open = !this.handoffs.history_open;
    cx.notify();
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
        let board = &self.handoffs.board;
        if devices || (board.active.is_empty() && board.history.is_empty()) {
            return None;
        }
        let font = &self.config.sidebar;
        let theme = &self.theme;
        let body = (!self.handoffs.collapsed).then(|| {
            let active: Vec<_> = board
                .active
                .iter()
                .enumerate()
                .map(|(index, handoff)| self.row(("handoff-row", index), handoff, None, look, cx))
                .collect();
            let shelved: Vec<_> = if self.handoffs.history_open {
                board
                    .history
                    .iter()
                    .enumerate()
                    .map(|(index, shelved)| {
                        self.row(
                            ("handoff-history-row", index),
                            &shelved.handoff,
                            Some(shelved.shelf),
                            look,
                            cx,
                        )
                    })
                    .collect()
            } else {
                Vec::new()
            };
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
                .children(active)
                .children(shelved)
        });
        let count = div()
            .debug_selector(|| "handoffs-count".into())
            .child(label_text(&board.active.len().to_string()));
        let right = div()
            .flex()
            .items_center()
            .gap_2()
            .child(self.history_toggle(cx))
            .child(count);
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
                    header_frame("handoffs", font, theme, look)
                        .justify_between()
                        .child(self.title_toggle(look, cx))
                        .child(right),
                )
                .children(body),
        )
    }

    /// The title and its chevron, which fold the body. Its own click target,
    /// apart from the history control beside it.
    fn title_toggle(&self, look: SidebarLook, cx: &mut Context<Self>) -> Stateful<Div> {
        let folded = self.handoffs.collapsed;
        let title = div()
            .id("handoffs-toggle")
            .debug_selector(|| "handoffs-toggle".into())
            .role(Role::Button)
            .aria_label(if folded {
                "Expand handoffs"
            } else {
                "Collapse handoffs"
            })
            .flex()
            .items_center()
            .gap_1()
            .cursor_pointer()
            .hover(|style| style.text_color(rgb(self.theme.foreground)))
            .on_click(cx.listener(|this, _, _, cx| {
                cx.stop_propagation();
                flip_folded(this, cx);
            }))
            .child(
                div()
                    .debug_selector(|| "handoffs-chevron".into())
                    .child(label_text(if folded {
                        CHEVRON_FOLDED
                    } else {
                        CHEVRON_OPEN
                    })),
            )
            .child(
                div()
                    .debug_selector(|| "header-label-handoffs".into())
                    .child(label_text(&look.header_label("handoffs"))),
            );
        let title = self.chrome_stop(Region::Sidebar, title);
        expandable(title, !folded, cx.entity().downgrade(), |this, _, cx| {
            flip_folded(this, cx)
        })
    }

    /// The control that shows or hides the picked-up and superseded rows.
    fn history_toggle(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let open = self.handoffs.history_open;
        let count = self.handoffs.board.history.len();
        let control = div()
            .id("handoffs-history-toggle")
            .debug_selector(|| "handoffs-history-toggle".into())
            .role(Role::Button)
            .aria_label(format!("Picked up and superseded handoffs: {count}"))
            .cursor_pointer()
            .text_color(rgb(self.theme.muted))
            .hover(|style| style.text_color(rgb(self.theme.foreground)))
            .on_click(cx.listener(|this, _, _, cx| {
                cx.stop_propagation();
                flip_history(this, cx);
            }))
            .child(label_text(&format!("picked up {count}")));
        let control = self.chrome_stop(Region::Sidebar, control);
        expandable(control, open, cx.entity().downgrade(), |this, _, cx| {
            flip_history(this, cx);
        })
    }

    /// Puts the handoff file's absolute path on the clipboard and flashes it,
    /// so the operator can paste it into a session.
    fn copy_handoff_path(&mut self, path: &Path, cx: &mut Context<Self>) {
        let text = path.to_string_lossy().into_owned();
        cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
        self.show_flash(Flash::success(format!("Copied {text}")), cx);
    }

    /// One handoff row. A click copies its path; a right-click opens the file
    /// in a code tab. A history row is dimmer and names why it is there.
    fn row(
        &self,
        slot: (&'static str, usize),
        handoff: &Handoff,
        shelf: Option<Shelf>,
        look: SidebarLook,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let (prefix, index) = slot;
        let theme = &self.theme;
        let path: PathBuf = handoff.path.clone();
        let target = target_for(handoff);
        let colors = row_colors(theme, shelf);
        let aria = match shelf {
            None => format!("Copy path of handoff {}", handoff.id),
            Some(shelf) => format!("Copy path of {} handoff {}", shelf.label(), handoff.id),
        };
        let row = div()
            .id(slot)
            .debug_selector(|| format!("{prefix}-{index}"))
            .role(Role::Button)
            .aria_label(aria)
            .flex_none()
            .min_w_0()
            .px(px(look.content_x()))
            .py(px(look.row_padding()))
            .cursor_pointer()
            .when_some(colors.bg, |row, bg| row.bg(rgb(bg)))
            .map(|row| match colors.hover_bg {
                Some(hover) => row.hover(|row| row.bg(rgb(hover))),
                None => row.hover(|row| row.bg(rgba(crate::panel_resize::RESIZE_HOVER))),
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.copy_handoff_path(&path, cx);
            }))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, _: &MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    this.open_code_view(&target, window, cx);
                }),
            )
            .child(
                div()
                    .truncate()
                    .text_color(rgb(colors.id_text))
                    .child(label_text(&handoff.id)),
            )
            .child(
                div()
                    .truncate()
                    .text_color(rgb(colors.detail_text))
                    .child(label_text(&detail(handoff, shelf))),
            );
        self.chrome_stop(Region::Sidebar, row)
    }
}
