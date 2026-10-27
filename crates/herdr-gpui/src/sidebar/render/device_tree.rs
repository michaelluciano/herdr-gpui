//! The Devices layout's tree: under each device's header, that device's
//! agents rather than its workspaces, grouped by worktree and narrowed by a
//! search field above the list. The headers are the spaces list's own, so folding, menus, and the
//! pinned header work as in every other layout.

use super::worktree_group;
use crate::{
    HerdrWindow,
    sidebar::{
        Indicators,
        agents::sorted_agents,
        cell::RowContext,
        layout::SidebarLook,
        unified::{self, facts},
        wash,
    },
};
use gpui::{prelude::*, *};

impl HerdrWindow {
    /// What the Devices layout's search field holds.
    pub(in crate::sidebar) fn device_tree_query(&self, cx: &App) -> String {
        self.devices_overview
            .tree_search
            .as_ref()
            .map(|search| search.input.read(cx).text().to_owned())
            .unwrap_or_default()
    }

    /// Whether endpoint `index` is listed for `query`: its name matches, or
    /// one of its agents does. A header stays only while something under it
    /// matches.
    pub(in crate::sidebar) fn device_listed(&self, index: usize, query: &str) -> bool {
        if query.trim().is_empty() {
            return true;
        }
        let Some(endpoint) = self.endpoints.get(index) else {
            return false;
        };
        if unified::host_matches(&endpoint.label, query) {
            return true;
        }
        let live = if index == self.selected_endpoint {
            &self.live
        } else {
            &endpoint.live
        };
        live.snapshot.as_deref().is_some_and(|snapshot| {
            snapshot
                .agents
                .iter()
                .any(|agent| unified::matches(&facts::of(&endpoint.id, agent, snapshot), query))
        })
    }

    /// Appends endpoint `index`'s agents, nested under its header and grouped
    /// by worktree, and returns the list with how many rows it added and
    /// which of them is the focused agent, for the reveal.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn append_device_agents(
        &self,
        mut list: Stateful<Div>,
        index: usize,
        indicators: Indicators,
        look: SidebarLook,
        width: f32,
        cx: &mut Context<Self>,
    ) -> (Stateful<Div>, usize, Option<usize>) {
        let Some(endpoint) = self.endpoints.get(index) else {
            return (list, 0, None);
        };
        if endpoint.collapsed {
            return (list, 0, None);
        }
        let selected = index == self.selected_endpoint;
        let live = if selected { &self.live } else { &endpoint.live };
        let Some(snapshot) = live
            .snapshot
            .as_deref()
            .filter(|_| live.status.is_connected())
        else {
            return (list, 0, None);
        };
        let theme = &self.theme;
        let guide = theme.tokens().line;
        let mark =
            wash::HostMark::resolve(&self.config.sidebar_style, &endpoint.label, selected, theme);
        let row_cx = |nest: f32, width: f32| RowContext {
            indicators,
            font: &self.config.sidebar,
            theme,
            look,
            width,
            nest,
            mark,
            // The header above already names the host.
            host: None,
        };
        let flat = row_cx(look.nest_indent(), width);
        // A group's sessions step in once more, under a guide line.
        let step = look.nest_indent();
        let grouped = row_cx(look.nest_indent(), width - step);
        // Configured `[sidebar_layout.agents]` rows apply here as in the
        // agents panel, gap included.
        let shown: Vec<_> = sorted_agents(snapshot, self.agent_sort)
            .into_iter()
            .filter_map(|agent| {
                let lines = self.configured_agent_lines(agent, snapshot, None)?;
                Some((agent, lines, facts::of(&endpoint.id, agent, snapshot)))
            })
            .collect();
        let all_facts: Vec<_> = shown.iter().map(|(.., facts)| facts.clone()).collect();
        let query = self.device_tree_query(cx);
        let entries = unified::arrange(
            &all_facts,
            &query,
            unified::host_matches(&endpoint.label, &query),
            &self.collapsed_worktree_groups,
        );
        let mut count = 0;
        let mut focused = None;
        let mut agent_row = |list: Stateful<Div>,
                             at: usize,
                             nested: bool,
                             count: &mut usize,
                             cx: &mut Context<Self>| {
            let (agent, configured, facts) = &shown[at];
            let lines = if configured.is_empty() {
                worktree_group::session_lines(facts, theme)
            } else {
                configured.clone()
            };
            if selected && agent.focused {
                focused = Some(*count);
            }
            let gap = self.agent_row_gap(*count);
            *count += 1;
            let context = if nested { &grouped } else { &flat };
            let cell = self.agent_cell(index, agent, snapshot, context, lines, gap, false, cx);
            list.child(if nested {
                cell.ml(px(look.content_x() + step))
                    .border_l_1()
                    .border_color(rgb(guide))
            } else {
                cell
            })
        };
        for entry in &entries {
            match entry {
                unified::Entry::Row(at) => list = agent_row(list, *at, false, &mut count, cx),
                unified::Entry::Group(group) => {
                    list = list.child(self.worktree_group_header(&endpoint.id, group, &flat, cx));
                    count += 1;
                    if !group.collapsed {
                        for &at in &group.members {
                            list = agent_row(list, at, true, &mut count, cx);
                        }
                    }
                }
            }
        }
        if count == 0 {
            list = list.child(
                div()
                    .pl(px(look.content_x() + look.nest_indent()))
                    .text_color(rgb(theme.muted))
                    .child(crate::sidebar::label_text(if query.trim().is_empty() {
                        "no agents"
                    } else {
                        "no matches"
                    })),
            );
            count = 1;
        }
        (list, count, focused)
    }
}
