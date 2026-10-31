//! The commit rows of the graph list.
use super::{ROW_HEIGHT, paint};
use crate::{
    HerdrWindow,
    browser::{Slot, TabId},
    graph_view::geometry::{lane_metrics, row_geometry},
};
use gpui::{prelude::*, *};
use herdr_vcs::graph::{Commit, Ref};

/// A ref's label and its palette index (bright ANSI color).
pub(super) fn ref_chip(reference: &Ref) -> Option<(String, usize)> {
    match reference {
        Ref::Head => Some(("HEAD".to_owned(), 13)),
        Ref::LocalBranch(name) => Some((name.clone(), 10)),
        Ref::RemoteBranch { name, remote } => Some((format!("{remote}/{name}"), 12)),
        Ref::Tag(name) => Some((name.clone(), 11)),
        Ref::Other => None,
    }
}

/// The first seven characters of an object name.
pub(super) fn short_id(id: &str) -> String {
    id.chars().take(7).collect()
}

/// "3 h ago" for a commit's author time, `now_ms` being the clock.
pub(super) fn when(time: i64, now_ms: u64) -> String {
    let then_ms = u64::try_from(time).unwrap_or(0).saturating_mul(1000);
    crate::remote::ago(then_ms, now_ms)
}

pub(super) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

/// Background of a commit row: the sidebar's selected fill when selected.
pub(in crate::graph_view) fn row_background(selected: bool, theme: &crate::config::Theme) -> u32 {
    if selected {
        theme.tokens().selected
    } else {
        theme.background
    }
}

/// Color of the accent edge on the selected row, as in the sidebar.
pub(in crate::graph_view) fn selected_edge_color(theme: &crate::config::Theme) -> u32 {
    theme.tokens().accent
}

/// Width of the selected row's accent edge; the sidebar's `SELECTED_EDGE`.
const SELECTED_EDGE: f32 = 2.0;

impl HerdrWindow {
    /// The rows of `range` for the graph list of tab `id`.
    pub(super) fn graph_rows(
        &mut self,
        id: TabId,
        slot: Slot,
        range: std::ops::Range<usize>,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let theme = &self.theme;
        let Some(state) = self.graphs.get(&id) else {
            return Vec::new();
        };
        let (lane_width, gutter) = lane_metrics(state.layout.lane_count);
        let now = now_ms();
        let mut rows = Vec::with_capacity(range.len());
        for index in range {
            let (Some(commit), Some(row)) =
                (state.commits.get(index), state.layout.rows.get(index))
            else {
                continue;
            };
            let selected = state.selected.as_deref() == Some(commit.id.as_str());
            let segments = row_geometry(row, lane_width, ROW_HEIGHT);
            let commit_id = commit.id.clone();
            rows.push(
                div()
                    .id(SharedString::from(
                        slot.selector(&format!("graph-row-id-{index}")),
                    ))
                    .debug_selector(move || slot.selector(&format!("graph-row-{index}")))
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap_2()
                    .w_full()
                    .h(px(ROW_HEIGHT))
                    .px_2()
                    .relative()
                    .bg(rgb(row_background(selected, theme)))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if let Some(focus) = this.graph_focus.get(&id) {
                            window.focus(focus, cx);
                        }
                        if let Some(state) = this.graphs.get_mut(&id)
                            && state.select(&commit_id)
                        {
                            this.scroll_graph_to_selection(id);
                            this.load_graph_files(id, commit_id.clone(), cx);
                            cx.notify();
                        }
                    }))
                    .children(selected.then(|| {
                        div()
                            .absolute()
                            .left_0()
                            .top_0()
                            .bottom_0()
                            .w(px(SELECTED_EDGE))
                            .bg(rgb(selected_edge_color(theme)))
                            .debug_selector(move || slot.selector("graph-row-selected-edge"))
                    }))
                    .child(
                        div()
                            .relative()
                            .flex_none()
                            .w(px(gutter))
                            .h_full()
                            .debug_selector(move || slot.selector("graph-gutter"))
                            .child(paint::gutter_canvas(segments, theme)),
                    )
                    .children(chips(commit, theme))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .child(commit.subject.clone()),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_color(rgb(theme.muted))
                            .child(short_id(&commit.id)),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_color(rgb(theme.muted))
                            .child(commit.author.clone()),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_color(rgb(theme.muted))
                            .child(when(commit.time, now)),
                    )
                    .into_any_element(),
            );
        }
        rows
    }
}

fn chips(commit: &Commit, theme: &crate::config::Theme) -> Vec<Div> {
    commit
        .refs
        .iter()
        .filter_map(ref_chip)
        .map(|(label, color)| {
            let color = theme
                .palette
                .get(color)
                .copied()
                .unwrap_or(theme.foreground);
            div()
                .flex_none()
                .px_1()
                .rounded_sm()
                .border_1()
                .border_color(rgb(color))
                .text_color(rgb(color))
                .child(label)
        })
        .collect()
}
