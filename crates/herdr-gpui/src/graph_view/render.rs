//! Drawing a graph tab: a virtualised list of commit rows over the detail
//! of the selected commit. Reads the tab's prepared [`GraphState`] only.
use crate::{
    HerdrWindow,
    browser::{Slot, Tab},
    fonts::StyledFont,
};
use gpui::{prelude::*, *};

mod detail;
mod files;
mod paint;
pub(super) mod row;

/// Height of every commit row; rows sit flush against each other.
pub(super) const ROW_HEIGHT: f32 = 24.0;

impl HerdrWindow {
    /// A graph tab, drawn in `slot` where a page would be.
    pub(crate) fn render_graph_tab(
        &self,
        slot: Slot,
        tab: &Tab,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (theme, ui) = (&self.theme, &self.config.ui);
        let id = tab.id;
        let focus = self.graph_focus.get(&id);
        let focused = focus.is_some_and(|focus| focus.contains_focused(window, cx));
        let body = match self.graphs.get(&id) {
            Some(state) if !state.commits.is_empty() => {
                let count = state.row_count();
                let mut list = uniform_list(
                    SharedString::from(slot.selector("graph-list")),
                    count,
                    cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                        this.graph_rows(id, slot, range, cx)
                    }),
                )
                .flex_1()
                .min_h_0();
                if let Some(scroll) = self.graph_scroll.get(&id) {
                    list = list.track_scroll(scroll);
                }
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .child(list)
                    .child(self.graph_detail(slot, state))
                    .into_any_element()
            }
            _ => div()
                .id(SharedString::from(slot.selector("graph-empty-state")))
                .debug_selector(move || slot.selector("graph-empty"))
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(theme.muted))
                .child("No commits to show")
                .into_any_element(),
        };
        let mut root = div()
            .id(SharedString::from(slot.selector("graph-tab")))
            .debug_selector(move || slot.selector("graph-tab"))
            .relative()
            .size_full()
            .min_w_0()
            .bg(rgb(theme.background))
            .text_color(rgb(theme.foreground))
            .text_font(ui)
            .text_size(px(ui.size));
        if let Some(focus) = focus {
            root = root.track_focus(focus);
        }
        root.child(body)
            .when(focused, |root| {
                root.child(
                    div()
                        .absolute()
                        .size_0()
                        .debug_selector(move || slot.selector("graph-tab-focused")),
                )
            })
            .into_any_element()
    }
}
