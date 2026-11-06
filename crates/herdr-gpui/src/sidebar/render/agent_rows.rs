//! The agents list: every listed host's agents in the panel's one order, so a
//! blocked agent on one host can sit above an idle one on another.

use crate::{
    HerdrWindow, NavigationTarget,
    agent_activity::Tone,
    sidebar::{
        Indicators, agent_name,
        agents::{agent_place, pane_name, state_label, status_text, tab_position},
        cell::{AgentRow, Cell, RowContext, RowData, StatusIcon, layout_for},
        layout::SidebarLook,
        line_height, tokens, wash,
    },
};
use gpui::{prelude::*, *};
use herdr_client::protocol::{AgentStatus, ClientShellAgent, ClientShellSnapshot};

impl HerdrWindow {
    /// The letter and context of a local Claude agent's row, once its report
    /// named a transcript; other agents and hosts show neither.
    fn agent_stats_mark(
        &self,
        agent: &ClientShellAgent,
        endpoint_id: &str,
        theme: &crate::config::Theme,
    ) -> Option<crate::sidebar::agent_stats::StatsMark> {
        if endpoint_id != crate::endpoint::LOCAL || !crate::agent_activity::is_claude(agent) {
            return None;
        }
        let activity = &self.agent_activity;
        let stats = activity.stats(&agent.pane_id)?;
        Some(crate::sidebar::agent_stats::StatsMark::new(
            stats,
            activity.compact_limit(),
            theme,
        ))
    }

    /// Appends a row for each agent `panel_agents` lists, in that order, and
    /// returns the list with how many rows it painted and where the selected
    /// host's focused agent sits among them, for the reveal. A row's host
    /// shapes its own context, as the agent may belong to any listed host.
    pub(super) fn append_agent_rows(
        &self,
        mut list: Stateful<Div>,
        indicators: Indicators,
        look: SidebarLook,
        width: f32,
        cx: &mut Context<Self>,
    ) -> (Stateful<Div>, usize, Option<usize>) {
        let font = &self.config.sidebar;
        let theme = &self.theme;
        let multi = self.endpoints.len() > 1;
        let mut count = 0;
        let mut focused = None;
        for (index, agent) in self.panel_agents() {
            let Some(endpoint) = self.endpoints.get(index) else {
                continue;
            };
            let selected = index == self.selected_endpoint;
            let live = if selected { &self.live } else { &endpoint.live };
            let Some(snapshot) = live.snapshot.as_deref() else {
                continue;
            };
            let row_cx = RowContext {
                indicators,
                font,
                theme,
                look,
                width,
                // Agents list under their own heading, not under a host, but
                // each carries its host's colour.
                nest: 0.,
                mark: wash::HostMark::resolve(
                    &self.config.sidebar_style,
                    &endpoint.label,
                    selected,
                    theme,
                ),
                host: (multi && endpoint.id != crate::endpoint::LOCAL)
                    .then_some(endpoint.label.as_str()),
            };
            let Some(lines) = self.configured_agent_lines(agent, snapshot, row_cx.host) else {
                continue;
            };
            if selected && agent.focused {
                focused = Some(count);
            }
            let gap = self.agent_row_gap(count);
            count += 1;
            list =
                list.child(self.agent_cell(index, agent, snapshot, &row_cx, lines, gap, false, cx));
        }
        (list, count, focused)
    }

    /// Whether the agents' rows follow a configured `[sidebar_layout.agents]`.
    fn custom_agent_rows(&self) -> bool {
        self.config.usage.inline
            && self.config.sidebar_layout.agents != crate::config::AgentLayout::default()
    }

    /// The lines a configured agent row shows, empty for Herdr's own row, or
    /// None when the configured rows leave this agent out.
    pub(super) fn configured_agent_lines(
        &self,
        agent: &ClientShellAgent,
        snapshot: &ClientShellSnapshot,
        host: Option<&str>,
    ) -> Option<Vec<Vec<tokens::ResolvedToken>>> {
        if !self.custom_agent_rows() {
            return Some(Vec::new());
        }
        tokens::agent_rows(&self.config.sidebar_layout.agents, agent, snapshot, host)
    }

    /// Space above the agent row painted `count`-th in its list: a
    /// configured `row_gap` between configured rows, none otherwise.
    pub(super) fn agent_row_gap(&self, count: usize) -> f32 {
        if self.custom_agent_rows() && count > 0 {
            f32::from(self.config.sidebar_layout.agents.row_gap) * line_height(&self.config.sidebar)
        } else {
            0.
        }
    }

    /// One agent's row, for the agents list, under its device's header, or
    /// `nested` under its workspace, where the row above already names the
    /// place: the agent of endpoint `index`, drawn in `row_cx`, that shows
    /// its pane when clicked.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn agent_cell(
        &self,
        index: usize,
        agent: &ClientShellAgent,
        snapshot: &ClientShellSnapshot,
        row_cx: &RowContext,
        lines: Vec<Vec<tokens::ResolvedToken>>,
        gap: f32,
        nested: bool,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let rows = layout_for(self.config.layout.mode);
        let multi = self.endpoints.len() > 1;
        let selected = index == self.selected_endpoint;
        let endpoint_id = self
            .endpoints
            .get(index)
            .map(|endpoint| endpoint.id.clone())
            .unwrap_or_default();
        let id = agent.pane_id.clone();
        let navigate_endpoint = endpoint_id.clone();
        let menu_endpoint = endpoint_id.clone();
        let menu_pane = id.clone();
        let menu_open = selected && self.pane_menu_target() == Some(id.as_str());
        let name = agent_name(agent);
        let pane = pane_name(agent, snapshot, name);
        let place = (!nested).then(|| agent_place(agent, snapshot)).flatten();
        let activity = self.agent_activity_label(agent);
        let state = activity.as_ref().map_or_else(
            || state_label(agent, status_text(agent.agent_status)),
            |label| label.text.as_str().into(),
        );
        let status_color = activity.as_ref().map(|label| match label.tone {
            Tone::Working => row_cx.indicators.color(AgentStatus::Working),
            Tone::Waiting => row_cx.indicators.color(AgentStatus::Done),
            Tone::Blocked => row_cx.indicators.color(AgentStatus::Blocked),
            Tone::Muted => row_cx.indicators.color(AgentStatus::Idle),
            Tone::Subagents => row_cx.theme.ink(row_cx.theme.tokens().accent),
        });
        let spoken = crate::sidebar::accessibility::agent_name(
            name,
            pane.as_deref(),
            &state,
            place,
            row_cx.host,
        );
        Cell::new(
            rows,
            RowData::Agent(AgentRow {
                key: format!("agent-{id}"),
                name,
                icon: crate::icons::AgentIcon::from_identity(agent.agent.as_deref()),
                status: agent.agent_status,
                place,
                pane,
                tab: place.and_then(|_| tab_position(agent, snapshot)),
                status_icon: activity.as_ref().map(|label| StatusIcon {
                    glyph: label.glyph,
                    detail: label.detail.clone(),
                    words: label.text.clone(),
                }),
                // A Claude agent's activity label shows whatever the rows
                // configure, as it replaces nothing a layout asked for.
                status_text: (activity.is_some()
                    || self
                        .config
                        .sidebar_layout
                        .agents
                        .shows_status_text(agent.agent.as_deref()))
                .then_some(state),
                status_color,
                stats: self.agent_stats_mark(agent, &endpoint_id, row_cx.theme),
                lines,
            }),
            row_cx,
        )
        .selected(selected && agent.focused)
        .highlighted(menu_open)
        .row()
        .when(gap > 0., |row| row.mt(px(gap)))
        .id(SharedString::from(format!("agent-{endpoint_id}-{id}")))
        .role(Role::ListItem)
        .aria_label(spoken)
        .aria_selected(selected && agent.focused)
        .map(|control| self.chrome_stop(crate::window::chrome_focus::Region::Sidebar, control))
        .when(multi, |row| {
            row.debug_selector(|| format!("agent-{endpoint_id}-{id}"))
        })
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                this.open_agent_menu(&menu_endpoint, &menu_pane, event.position, window, cx);
            }),
        )
        .on_click(cx.listener(move |this, _, window, cx| {
            this.navigate_endpoint(&navigate_endpoint, NavigationTarget::Pane(&id), cx);
            window.focus(&this.focus, cx);
        }))
    }
}
