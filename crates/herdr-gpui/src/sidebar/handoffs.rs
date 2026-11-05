//! The Handoffs section below the workspaces and agents lists: the newest
//! unclaimed handoff of each project, and behind a header control every
//! handoff: picked up, superseded and dismissed. Each row has an x that
//! removes it from the list.

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

/// Keeps a row's remove control's element id apart from the row's own.
const REMOVE_OFFSET: usize = 1_000_000;
/// Keeps a row's lock control's element id apart from the others.
const PIN_OFFSET: usize = 2_000_000;
/// A closed padlock: the handoff is pinned.
const LOCK_CLOSED: &str = "icons/lock.svg";
/// An open padlock: the handoff is not pinned.
const LOCK_OPEN: &str = "icons/lock-open.svg";
/// The padlock's side, a little under the row's text height.
const LOCK_SIZE: f32 = 10.;
/// How faint the open padlock is until the pointer is over it.
const LOCK_OPEN_OPACITY: f32 = 0.45;

/// How quiet the header's `all N` toggle is until the pointer is over it.
const HISTORY_TOGGLE_OPACITY: f32 = 0.75;
/// Extra vertical padding of a row, for a little room between rows.
const ROW_GAP: f32 = 2.;

/// The least contrast a pinned padlock keeps against the sidebar (WCAG's
/// non-text minimum).
const LOCK_CONTRAST: f32 = 3.1;

/// The padlock's colour and opacity: the accent at full strength when pinned,
/// nudged in lightness only where a theme's accent falls under
/// [`LOCK_CONTRAST`] (Catppuccin Latte's pink does); the muted gray faded back
/// when not.
pub(super) fn pin_ink(theme: &Theme, pinned: bool) -> (u32, f32) {
    if pinned {
        let tokens = theme.tokens();
        let ink = crate::contrast::ink(
            tokens.accent,
            &[theme.background, tokens.rail],
            LOCK_CONTRAST,
        );
        (ink, 1.0)
    } else {
        (theme.muted, LOCK_OPEN_OPACITY)
    }
}

/// The colours of one handoff row.
pub(super) struct RowColors {
    pub(super) id_text: u32,
    pub(super) detail_text: u32,
}

/// An active row (`shelf == None`) sits on the sidebar's own background: the
/// id line in white (the foreground), the detail line a step dimmer (the
/// foreground blended 12% toward the background, the nearest step that keeps 4.5 on every theme tested). A history row is gray (muted) on both lines.
pub(super) fn row_colors(theme: &Theme, shelf: Option<Shelf>) -> RowColors {
    match shelf {
        None => RowColors {
            id_text: theme.foreground,
            detail_text: mix(theme.background, theme.foreground, 88),
        },
        Some(_) => RowColors {
            id_text: theme.muted,
            detail_text: theme.muted,
        },
    }
}

fn flip_folded(this: &mut HerdrWindow, cx: &mut Context<HerdrWindow>) {
    this.handoffs.collapsed = !this.handoffs.collapsed;
    cx.notify();
}

fn dismiss(this: &mut HerdrWindow, path: PathBuf, cx: &mut Context<HerdrWindow>) {
    this.handoffs.dismiss(path);
    cx.notify();
}

fn toggle_pin(this: &mut HerdrWindow, path: PathBuf, cx: &mut Context<HerdrWindow>) {
    this.handoffs.toggle_pin(path);
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
            let bar = self
                .handoffs
                .history_open
                .then(|| self.bulk_actions(cx))
                .flatten();
            let list = div()
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
                .children(shelved);
            div().min_h_0().flex().flex_col().children(bar).child(list)
        });
        let count = div()
            .debug_selector(|| "handoffs-count".into())
            .text_color(rgb(theme.muted))
            .child(label_text(&format!("\u{b7} {} active", board.active.len())));
        let left = div()
            .flex()
            .items_center()
            .gap_1()
            .child(self.title_toggle(look, cx))
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
                        .child(left)
                        .child(self.history_toggle(cx)),
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

    /// The all view's bar: clear the picked-up rows, restore the dismissed
    /// ones. `None` when neither has anything to act on.
    fn bulk_actions(&self, cx: &mut Context<Self>) -> Option<Div> {
        let picked = self.handoffs.board.count(Shelf::PickedUp);
        let dismissed = self.handoffs.board.count(Shelf::Dismissed);
        if picked == 0 && dismissed == 0 {
            return None;
        }
        let action = |id: &'static str, text: String, aria: String| {
            div()
                .id(id)
                .debug_selector(|| id.into())
                .role(Role::Button)
                .aria_label(aria)
                .cursor_pointer()
                .text_color(rgb(self.theme.muted))
                .hover(|style| style.text_color(rgb(self.theme.foreground)))
                .child(label_text(&text))
        };
        let clear = (picked > 0).then(|| {
            let control = action(
                "handoffs-clear-picked-up",
                format!("Clear picked up ({picked})"),
                format!("Remove {picked} picked up handoffs from the list"),
            )
            .on_click(cx.listener(|this, _, _, cx| {
                cx.stop_propagation();
                this.handoffs.clear_picked_up();
                cx.notify();
            }));
            self.chrome_stop(Region::Sidebar, control)
        });
        let restore = (dismissed > 0).then(|| {
            let control = action(
                "handoffs-restore",
                format!("Restore cleared ({dismissed})"),
                format!("Restore {dismissed} removed handoffs to the list"),
            )
            .on_click(cx.listener(|this, _, _, cx| {
                cx.stop_propagation();
                this.handoffs.restore_dismissed();
                cx.notify();
            }));
            self.chrome_stop(Region::Sidebar, control)
        });
        Some(
            div()
                .flex_none()
                .flex()
                .items_center()
                .gap_3()
                .px(px(self.config.sidebar.size))
                .py_1()
                .children(clear)
                .children(restore),
        )
    }

    /// The control that shows or hides every handoff beyond the active ones.
    fn history_toggle(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let open = self.handoffs.history_open;
        let count = self.handoffs.board.history.len();
        let control = div()
            .id("handoffs-history-toggle")
            .debug_selector(|| "handoffs-history-toggle".into())
            .role(Role::Button)
            .aria_label(format!("All handoffs: {count}"))
            .cursor_pointer()
            .text_color(rgb(self.theme.muted))
            .opacity(HISTORY_TOGGLE_OPACITY)
            .hover(|style| style.text_color(rgb(self.theme.foreground)).opacity(1.0))
            .on_click(cx.listener(|this, _, _, cx| {
                cx.stop_propagation();
                flip_history(this, cx);
            }))
            .child(label_text(&format!("all {count}")));
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
        let remove_path = path.clone();
        let pin_path = path.clone();
        let pinned = self.handoffs.pinned.contains(&path);
        let target = target_for(handoff);
        let colors = row_colors(theme, shelf);
        let aria = match shelf {
            None => format!("Copy path of handoff {}", handoff.id),
            Some(shelf) => format!("Copy path of {} handoff {}", shelf.label(), handoff.id),
        };
        let (glyph, pin_aria) = if pinned {
            (LOCK_CLOSED, format!("Unpin handoff {}", handoff.id))
        } else {
            (LOCK_OPEN, format!("Pin handoff {}", handoff.id))
        };
        let (ink, opacity) = pin_ink(theme, pinned);
        // Centred on the row's first line, whatever the row's height.
        let pin = div()
            .id((prefix, index + PIN_OFFSET))
            .debug_selector(|| format!("{prefix}-pin-{index}"))
            .role(Role::Button)
            .aria_label(pin_aria)
            .flex_none()
            .self_start()
            .h(px(line_height(&self.config.sidebar)))
            .flex()
            .items_center()
            .px(px(2.))
            .cursor_pointer()
            .text_color(rgb(ink))
            .opacity(opacity)
            .hover(|style| style.text_color(rgb(theme.foreground)).opacity(1.0))
            .on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                toggle_pin(this, pin_path.clone(), cx);
            }))
            // The svg takes its tint from the control, so the hover can change it.
            .child(svg().path(glyph).size(px(LOCK_SIZE)).flex_none());
        let pin = self.chrome_stop(Region::Sidebar, pin);
        let group = SharedString::from(format!("{prefix}-group-{index}"));
        // Hidden by opacity, not removed: it keeps its width and stays
        // reachable by keyboard and assistive tech.
        let remove = div()
            .id((prefix, index + REMOVE_OFFSET))
            .debug_selector(|| format!("{prefix}-x-{index}"))
            .role(Role::Button)
            .aria_label(format!("Remove handoff {} from list", handoff.id))
            .flex_none()
            .px_1()
            .cursor_pointer()
            .text_color(rgb(colors.detail_text))
            .opacity(0.)
            .group_hover(group.clone(), |style| style.opacity(1.0))
            .hover(|style| style.text_color(rgb(colors.id_text)))
            .on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                dismiss(this, remove_path.clone(), cx);
            }))
            .child(label_text("\u{d7}"));
        let remove = self.chrome_stop(Region::Sidebar, remove);
        let row =
            div()
                .id(slot)
                .group(group)
                .debug_selector(|| format!("{prefix}-{index}"))
                .role(Role::Button)
                .aria_label(aria)
                .flex_none()
                .min_w_0()
                .px(px(look.content_x()))
                .py(px(look.row_padding() + ROW_GAP))
                .cursor_pointer()
                .hover(|row| row.bg(rgba(crate::panel_resize::RESIZE_HOVER)))
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
                .flex()
                .items_center()
                .gap_1()
                .child(pin)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(
                            div()
                                .truncate()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(colors.id_text))
                                .child(label_text(&handoff.project)),
                        )
                        .child(div().truncate().text_color(rgb(colors.detail_text)).child(
                            label_text(&detail(handoff, shelf, chrono::Local::now().naive_local())),
                        )),
                )
                .child(remove);
        self.chrome_stop(Region::Sidebar, row)
    }
}
