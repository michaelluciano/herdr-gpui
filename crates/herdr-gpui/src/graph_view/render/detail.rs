//! The detail pane under the list: the selected commit in full.
use super::row::{now_ms, when};
use crate::{
    HerdrWindow,
    browser::Slot,
    graph_view::{render::row::ref_chip, state::GraphState},
};
use gpui::{prelude::*, *};

/// The pane never gets shorter than this, however short the tab.
const DETAIL_MIN_HEIGHT: f32 = 140.0;
/// Share of the tab the pane takes when that is above the floor.
const DETAIL_SHARE: f32 = 0.38;

impl HerdrWindow {
    /// The pane for the selected commit of `state`; empty when none is.
    pub(super) fn graph_detail(&self, slot: Slot, state: &GraphState) -> Stateful<Div> {
        let theme = &self.theme;
        let pane = div()
            .id(SharedString::from(slot.selector("graph-detail-pane")))
            .debug_selector(move || slot.selector("graph-detail"))
            .flex_none()
            .w_full()
            .h(relative(DETAIL_SHARE))
            .min_h(px(DETAIL_MIN_HEIGHT))
            .p_2()
            .overflow_hidden()
            .border_t_1()
            .border_color(rgb(theme.surface))
            .bg(rgb(theme.surface));
        let Some(commit) = state
            .selected_index()
            .and_then(|index| state.commits.get(index))
        else {
            return pane;
        };
        let refs: Vec<String> = commit
            .refs
            .iter()
            .filter_map(ref_chip)
            .map(|(label, _)| label)
            .collect();
        let line = |label: &'static str, value: String| {
            div()
                .flex()
                .gap_2()
                .child(
                    div()
                        .flex_none()
                        .w(px(64.0))
                        .text_color(rgb(theme.muted))
                        .child(label),
                )
                .child(div().flex_1().min_w_0().child(value))
        };
        let name = format!("graph-detail-{}", commit.id);
        pane.child(
            div()
                .size_full()
                .flex()
                .flex_col()
                .debug_selector(move || slot.selector(&name))
                .child(line("Commit", commit.id.clone()))
                .child(line("Author", commit.author.clone()))
                .child(line("When", when(commit.time, now_ms())))
                .child(line("Subject", commit.subject.clone()))
                .child(line("Parents", commit.parents.join(" ")))
                .child(line("Refs", refs.join(", ")))
                .child(
                    div()
                        .id(SharedString::from(slot.selector("graph-files-scroll-area")))
                        .debug_selector(move || slot.selector("graph-files-scroll"))
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(self.graph_files(slot, state, &commit.id)),
                ),
        )
    }
}
