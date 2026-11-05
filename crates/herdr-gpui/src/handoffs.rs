//! The Handoffs sidebar section: the session handoff files under
//! `$HOME/claude-private/handoffs`, read off the UI thread every few seconds
//! and sorted into the active list and the history. File contents are
//! untrusted text; only a few front matter keys are read.

mod classify;
mod line;
mod list;
mod poll;
mod target;

#[cfg(test)]
mod tests;

pub(crate) use classify::{Shelf, classify};
pub(crate) use line::{detail, title};
pub(crate) use list::Handoff;
#[cfg(test)]
pub(crate) use list::Kind;
#[cfg(test)]
pub(crate) use list::list_all;
pub(crate) use poll::Handoffs;
#[cfg(test)]
pub(crate) use poll::root_under;
pub(crate) use target::target_for;
