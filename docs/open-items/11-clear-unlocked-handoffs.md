# Clear handoffs that are not locked

Status: not started; requested 2026-11-08

## Goal

Give the Handoffs panel a way to clear every handoff that is not locked, so finished and superseded handoffs stop piling up in the list.

## Decisions already made

- The action clears only handoffs that are not locked; a locked handoff always stays.

## Open questions

- What "locked" means for a handoff. The session lock from PR 27 locks the agent panel's order and does not apply to handoff rows, so a per-handoff lock would be new: where it is stored (in the handoff root, so every client agrees, or in the app) and how a row shows and toggles it.
- What "clear" does to the file: delete it, move it into `history/`, or only hide the row. Handoffs are files under the handoff root (`LATEST.md`, `sessions/`, `history/`), so a delete cannot be undone from the app.
- Whether it asks for confirmation. The repository rules say destructive actions open the app's own dialog and the user confirms there.
- Whether it applies to all projects or to the focused project only, and whether it belongs on the control socket (`docs/cli.md`) as well as in the panel.

## Where to start

- Handoffs panel: `crates/herdr-gpui/src/sidebar/handoffs.rs`; the list and its kinds are in `crates/herdr-gpui/src/handoffs/list.rs`; the refresh is `handoffs/poll.rs` (a changed file shows within about a second, PR 35).
- Do the file work on the background executor, never on the UI thread.
