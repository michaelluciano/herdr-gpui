//! The Handoffs sidebar section: the open session handoff files under
//! `$HOME/claude-private/handoffs`, read off the UI thread every few seconds.
//! File contents are untrusted text; only a few front matter keys are read.

mod list;
mod poll;
mod target;

#[cfg(test)]
mod tests;

pub(crate) use list::Handoff;
#[cfg(test)]
pub(crate) use list::list_open;
pub(crate) use poll::Handoffs;
#[cfg(test)]
pub(crate) use poll::root_under;
pub(crate) use target::target_for;
