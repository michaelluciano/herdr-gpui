//! Loading the graph and the selected commit's files off the UI thread.
use super::state::GraphState;
use crate::{
    HerdrWindow,
    browser::{Location, Store, TabId},
};
use gpui::Context;
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

/// How often an open graph tab reloads its commits.
pub(crate) const GRAPH_REFRESH: Duration = Duration::from_secs(5);
/// Most commits one load reads.
pub(crate) const GRAPH_LIMIT: u32 = 500;
/// Longest one Git read may take.
const GRAPH_DEADLINE: Duration = Duration::from_secs(10);

impl HerdrWindow {
    /// The local checkout directory a graph tab shows, if it has one.
    fn graph_checkout(cx: &Context<Self>, id: TabId) -> Option<String> {
        match cx.try_global::<Store>()?.get(id)?.location.as_ref()? {
            Location::Graph { checkout } => checkout.checkout.clone(),
            _ => None,
        }
    }

    /// Starts a commit load for the tab on the background executor; the
    /// result lands through [`GraphState::apply`]. A failed load changes
    /// nothing; the attempt is still marked, so the next try waits a refresh.
    pub(crate) fn load_graph(&mut self, id: TabId, cx: &mut Context<Self>) {
        let Some(checkout) = Self::graph_checkout(cx, id) else {
            return;
        };
        let Some(state) = self.graphs.get_mut(&id) else {
            return;
        };
        let (request, flag) = state.begin_request();
        let loading = cx.background_executor().spawn(async move {
            herdr_vcs::graph::log(
                &checkout,
                GRAPH_LIMIT,
                Instant::now() + GRAPH_DEADLINE,
                &|| flag.load(Ordering::SeqCst),
            )
        });
        cx.spawn(async move |this, cx| {
            let result = loading.await;
            this.update(cx, |this, cx| {
                let Ok(commits) = result else {
                    return;
                };
                let Some(state) = this.graphs.get_mut(&id) else {
                    return;
                };
                if state.apply(request, commits) {
                    this.scroll_graph_to_selection(id);
                    cx.notify();
                }
                this.load_missing_graph_files(id, cx);
            })
            .ok();
        })
        .detach();
    }

    /// Loads the selected commit's files when they are not there yet.
    fn load_missing_graph_files(&mut self, id: TabId, cx: &mut Context<Self>) {
        let Some(state) = self.graphs.get(&id) else {
            return;
        };
        let Some(selected) = state.selected.clone() else {
            return;
        };
        if state.files.as_ref().map(|files| files.commit.as_str()) != Some(selected.as_str()) {
            self.load_graph_files(id, selected, cx);
        }
    }

    /// Starts a changed-files load of `commit` for the tab's detail pane.
    pub(crate) fn load_graph_files(&mut self, id: TabId, commit: String, cx: &mut Context<Self>) {
        let Some(checkout) = Self::graph_checkout(cx, id) else {
            return;
        };
        let Some(state) = self.graphs.get_mut(&id) else {
            return;
        };
        let (request, flag) = state.begin_files_request();
        let hash = commit.clone();
        let loading = cx.background_executor().spawn(async move {
            herdr_vcs::graph::changed_files(
                &checkout,
                &hash,
                Instant::now() + GRAPH_DEADLINE,
                &|| flag.load(Ordering::SeqCst),
            )
        });
        cx.spawn(async move |this, cx| {
            let result = loading.await;
            this.update(cx, |this, cx| {
                let Ok(changes) = result else {
                    return;
                };
                if let Some(state) = this.graphs.get_mut(&id)
                    && state.apply_files(request, &commit, changes)
                {
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    /// Starts a load for each graph tab of the focused workspace that is due.
    pub(crate) fn poll_graphs_at(&mut self, now: Instant, cx: &mut Context<Self>) {
        let Some((scope, workspace)) = self.browser_key() else {
            return;
        };
        let Some(store) = cx.try_global::<Store>() else {
            return;
        };
        let due: Vec<TabId> = store
            .in_workspace(&scope, &workspace)
            .filter(|tab| matches!(tab.location, Some(Location::Graph { .. })))
            .map(|tab| tab.id)
            .filter(|id| {
                self.graphs
                    .get(id)
                    .is_some_and(|state: &GraphState| state.refresh_due(now, GRAPH_REFRESH))
            })
            .collect();
        for id in due {
            self.load_graph(id, cx);
        }
    }
}
