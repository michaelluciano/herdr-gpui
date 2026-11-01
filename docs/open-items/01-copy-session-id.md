# Copy session id

Status: not started

## Goal

Let the operator copy an agent session's full id from the sidebar so it can be pasted into a resume command or a note.

## Decisions already made

- The only entry point is a right-click menu entry on session (agent) rows. No hover tooltip and no inline short id.
- A workspace row that has an agent gets one entry per agent, labelled like `claude · 184567ac` (agent name, then the first eight characters of the id).
- The data comes from `pane.agent_session`, the `teleport::snapshot::AgentSession` type in `crates/herdr-remote/src/teleport/snapshot.rs`. `crates/herdr-gpui/src/chat/existing.rs` already reads it.
- Choosing an entry copies the full id and flashes it so the operator sees what was copied.
- Tooltip precedent, for reference only: `crates/herdr-gpui/src/sidebar/host_row.rs` line 33.
- Work goes on its own branch, `feat/session-id-copy`, with its own pull request.

## Open questions

- Which sidebar row types already have a context menu to extend, and which need one added.
- What the label shows when an agent has no session id yet (hide the entry or show it disabled).

## Where to start

- `crates/herdr-gpui/src/chat/existing.rs` for how `agent_session` is read from a pane.
- `crates/herdr-gpui/src/sidebar/` for the row and menu code.
- Add a test that the entry list for a workspace with two agents has two labelled entries, and that the copied text is the full id.
