//! What a click on a row does, and the pieces it needs.
use super::{Blocker, RowState};
use crate::browser::WebUrl;
use crate::config::llm_launchers::Launcher;

/// What a click does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Plan {
    Open,
    StartThenOpen,
    /// Another row holds the group's slot.
    Refuse {
        by: Blocker,
    },
    Ignore,
}

pub(crate) fn plan(state: RowState) -> Plan {
    match state {
        RowState::Running => Plan::Open,
        RowState::Stopped => Plan::StartThenOpen,
        RowState::Blocked { by } => Plan::Refuse { by },
        RowState::Starting => Plan::Ignore,
    }
}

/// Where a running row opens: its `url`, else the loopback `port`.
pub(crate) fn url_for(port: u16, url: Option<&str>) -> crate::Result<WebUrl> {
    match url {
        Some(url) => WebUrl::from_typed(url),
        None => WebUrl::from_typed(&format!("http://127.0.0.1:{port}/")),
    }
}

/// The rows on one endpoint, each with its index in `launchers`.
pub(crate) fn rows_for_host<'a>(
    launchers: &'a [Launcher],
    endpoint_label: &str,
    endpoint_id: &str,
) -> Vec<(usize, &'a Launcher)> {
    launchers
        .iter()
        .enumerate()
        .filter(|(_, launcher)| launcher.on_host(endpoint_label, endpoint_id))
        .collect()
}
