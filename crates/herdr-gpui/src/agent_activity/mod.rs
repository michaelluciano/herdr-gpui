//! What each Claude agent is doing, shown on its sidebar row. Claude Code's
//! hooks write one small state file per pane under
//! `$HOME/.local/state/herdr-claude/`; the window reads them off the UI thread
//! every two seconds and checks them against the daemon's own status. File
//! contents are untrusted text, bounded when read.

mod glyph;
mod label;
mod poll;
mod read;

#[cfg(test)]
mod tests;

pub(crate) use glyph::Glyph;
pub(crate) use label::Tone;
pub(crate) use poll::AgentActivity;
