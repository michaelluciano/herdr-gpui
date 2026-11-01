//! The LLM launcher rows' decisions: what each row shows and what a click
//! does. Pure: the probes, terminals, and drawing live elsewhere.
#![allow(unused_imports)] // re-exports the tests reach through `super::*`

mod click;
mod observe;
mod plan;
mod poll;
mod run;
mod state;
mod terminal;

pub(crate) use click::{Effects, starting_rows, states_with_starting};
pub(crate) use observe::{
    Observed, foreign_ports, needs_ps, observe, observe_with, parse_llama_ports, probe_health,
    run_ps, states,
};
pub(crate) use plan::{Plan, plan, rows_for_host, url_for};
pub(crate) use poll::Launchers;
pub(crate) use run::{StartError, start};
pub(crate) use state::{Blocker, Health, RowInput, RowState, health_from_status, row_states};
pub(crate) use terminal::{
    Gate, TerminalJob, click_gate, create_params, pick_workspace, resolve_endpoint,
};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
