//! Copying an agent session's full id from a sidebar menu row. The id is the
//! one chat discovery verified for the pane (`pane.agent_session`), since the
//! client snapshot carries none.

use crate::{HerdrWindow, chat::existing::Binding, window::Flash};
use gpui::{ClipboardItem, Context};
use herdr_client::protocol::ClientShellSnapshot;

/// The characters of the id a menu label shows; the copy takes all of it.
const SHORT_ID: usize = 8;

/// One agent session a menu can copy: the agent's name and the full id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Session {
    pub(crate) agent: &'static str,
    pub(crate) id: String,
}

impl Session {
    /// `claude \u{b7} 184567ac`: the agent, then the first eight characters of its id.
    pub(crate) fn label(&self) -> String {
        let short: String = self.id.chars().take(SHORT_ID).collect();
        format!("{} \u{b7} {short}", self.agent)
    }
}

fn session(binding: &Binding) -> Session {
    Session {
        agent: binding.agent.name(),
        id: binding.session_id.clone(),
    }
}

/// The session of `pane`, when chat discovery verified one for it.
pub(crate) fn of_pane(bindings: &[Binding], pane: &str) -> Option<Session> {
    bindings
        .iter()
        .find(|binding| binding.pane_id == pane)
        .map(session)
}

/// The sessions of `workspace`'s panes, in the snapshot's pane order.
pub(crate) fn of_workspace(
    snapshot: &ClientShellSnapshot,
    bindings: &[Binding],
    workspace: &str,
) -> Vec<Session> {
    snapshot
        .panes
        .iter()
        .filter(|pane| pane.workspace_id == workspace)
        .filter_map(|pane| of_pane(bindings, &pane.pane_id))
        .collect()
}

impl HerdrWindow {
    /// Puts the full id on the clipboard, and flashes it so the operator sees
    /// what was copied.
    pub(crate) fn copy_session_id(&mut self, id: &str, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(id.to_owned()));
        self.show_flash(Flash::success(format!("Copied session {id}")), cx);
    }
}

#[cfg(test)]
pub(crate) mod tests;
