//! The graph lives in a tab beside the workspace's terminals and pages. The
//! tab keeps the checkout it shows; its [`GraphState`] lives in the window,
//! made when the tab opens or is first seen again, and dropped with the tab.
use super::state::GraphState;
use crate::{
    HerdrWindow,
    browser::{Location, ReviewCheckout, Store, TabId},
    window::Flash,
};
use gpui::{Context, ScrollStrategy, Window};

impl HerdrWindow {
    /// Opens the focused checkout's graph in a tab of the focused workspace,
    /// or brings back the one already open there.
    pub(crate) fn open_graph(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(checkout) = self
            .git
            .tracked()
            .and_then(|input| ReviewCheckout::try_from(input).ok())
        else {
            self.show_flash(Flash::warning("No local checkout to show"), cx);
            return;
        };
        let Some((scope, workspace)) = self.browser_key() else {
            self.show_flash(Flash::warning("Open a workspace first"), cx);
            return;
        };
        let existing = cx.try_global::<Store>().and_then(|store| {
            store
                .in_workspace(&scope, &workspace)
                .find(|tab| {
                    matches!(&tab.location, Some(Location::Graph { checkout: open }) if *open == checkout)
                })
                .map(|tab| tab.id)
        });
        let opened = existing.or_else(|| {
            Store::update(cx, |store| {
                store.open(scope, &workspace, Some(Location::Graph { checkout }), None)
            })
        });
        let Some(id) = opened else {
            self.show_flash(Flash::warning("Too many tabs are open"), cx);
            return;
        };
        self.dismiss_menu(window, cx);
        self.graphs.entry(id).or_default();
        self.graph_scroll.entry(id).or_default();
        let focus = self
            .graph_focus
            .entry(id)
            .or_insert_with(|| cx.focus_handle())
            .clone();
        self.show_browser_tab(id, window, cx);
        window.focus(&focus, cx);
        cx.notify();
    }

    /// Scrolls the tab's list to the row the selection moved to, if it did.
    pub(crate) fn scroll_graph_to_selection(&mut self, id: TabId) {
        let target = self
            .graphs
            .get_mut(&id)
            .and_then(GraphState::take_scroll_target);
        if let (Some(index), Some(scroll)) = (target, self.graph_scroll.get(&id)) {
            scroll.scroll_to_item(index, ScrollStrategy::Nearest);
        }
    }

    /// Runs every window tick: graph tabs of the focused workspace without
    /// state, such as ones reopened after closing, get one, and the state of
    /// closed tabs goes.
    pub(crate) fn poll_graphs(&mut self, cx: &mut Context<Self>) {
        let Some(store) = cx.try_global::<Store>() else {
            self.graphs.clear();
            self.graph_focus.clear();
            self.graph_scroll.clear();
            return;
        };
        self.graphs.retain(|id, state: &mut GraphState| {
            let open = store.get(*id).is_some();
            if !open {
                state.cancel_all();
            }
            open
        });
        self.graph_focus.retain(|id, _| store.get(*id).is_some());
        self.graph_scroll.retain(|id, _| store.get(*id).is_some());
        let Some((scope, workspace)) = self.browser_key() else {
            return;
        };
        let missing: Vec<TabId> = store
            .in_workspace(&scope, &workspace)
            .filter(|tab| {
                matches!(tab.location, Some(Location::Graph { .. }))
                    && !self.graphs.contains_key(&tab.id)
            })
            .map(|tab| tab.id)
            .collect();
        for id in missing {
            self.graphs.entry(id).or_default();
            self.graph_focus
                .entry(id)
                .or_insert_with(|| cx.focus_handle());
            self.graph_scroll.entry(id).or_default();
        }
        self.poll_graphs_at(std::time::Instant::now(), cx);
    }
}
