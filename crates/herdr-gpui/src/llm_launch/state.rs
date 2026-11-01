//! What a row's health probe says and the state each row shows.
use crate::config::llm_launchers::canonical_host;

/// What a probe of a row's `/health` found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Health {
    /// HTTP 200.
    Ready,
    /// HTTP 503: the model is still loading.
    Loading,
    /// No answer.
    Down,
}

/// What holds a group's slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Blocker {
    /// A configured row, by index.
    Row(usize),
    /// A model no row names, listening on `port` of the same host.
    Foreign { port: u16 },
}

/// What a row shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RowState {
    Stopped,
    Starting,
    Running,
    /// Stopped, and another row of its host and group holds the slot.
    Blocked {
        by: Blocker,
    },
}

/// One row as the states are worked out.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RowInput<'a> {
    pub host: &'a str,
    pub group: Option<&'a str>,
    /// `None` for terminal rows and rows not probed.
    pub health: Option<Health>,
}

/// The state of every row, in input order. `foreign` is each model running
/// that no row names, as (host, port).
pub(crate) fn row_states(rows: &[RowInput], foreign: &[(&str, u16)]) -> Vec<RowState> {
    let base: Vec<RowState> = rows
        .iter()
        .map(|row| match row.health {
            Some(Health::Ready) => RowState::Running,
            Some(Health::Loading) => RowState::Starting,
            Some(Health::Down) | None => RowState::Stopped,
        })
        .collect();
    rows.iter()
        .zip(&base)
        .map(|(row, state)| {
            let (RowState::Stopped, Some(group)) = (state, row.group) else {
                return *state;
            };
            let host = canonical_host(row.host);
            let holder = rows.iter().zip(&base).position(|(other, other_state)| {
                matches!(other_state, RowState::Running | RowState::Starting)
                    && canonical_host(other.host) == host
                    && other.group == Some(group)
            });
            if let Some(index) = holder {
                return RowState::Blocked {
                    by: Blocker::Row(index),
                };
            }
            match foreign
                .iter()
                .find(|(foreign_host, _)| canonical_host(foreign_host) == host)
            {
                Some((_, port)) => RowState::Blocked {
                    by: Blocker::Foreign { port: *port },
                },
                None => RowState::Stopped,
            }
        })
        .collect()
}

/// A probe's HTTP status as a health.
pub(crate) fn health_from_status(status: Option<u16>) -> Health {
    match status {
        Some(200) => Health::Ready,
        Some(503) => Health::Loading,
        _ => Health::Down,
    }
}
