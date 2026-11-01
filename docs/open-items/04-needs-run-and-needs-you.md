# Needs-run command and "Needs you" section

Status: not started

## Goal

When a session needs the operator to run a command it cannot run itself, record it in one place and show it in the sidebar, instead of printing it into a terminal where long lines break. A `needs-run` command replaces a paste helper that is missing.

## Decisions already made

- `needs-run` writes one record file per session into a per-user pending directory.
- Record fields: the exact one-line command, host, working directory, why it is needed, the expected result, a risk tag (read-only, sudo prompt, destructive, classifier-blocked), and a blocks-session flag.
- A session registers once; asking again updates the same entry.
- Slash commands are shown as "type this in tab X" rather than as a runnable command.
- A "Needs you" sidebar section, built like the Handoffs section (`crates/herdr-gpui/src/sidebar/handoffs.rs`): polled off the UI thread, hidden when empty, rows grouped by session. Clicking a row copies the full command. A Done action clears it.
- "Run in new tab" opens a tab on that host and types the command without pressing Enter. It never runs anything on its own.
- An entry clears when the operator marks it done, when the session withdraws it, when the session stops, or when it is superseded by a re-register.

## Open questions

- Where the `needs-run` command itself lives (this repo or outside it).
- How a record is tied to a session so "session stop" can clear it.

## Later

- Overview page integration.
- Auto-clear when a matching `!` command appears in the session transcript.

## Where to start

- `crates/herdr-gpui/src/handoffs.rs` and `sidebar/handoffs.rs` for the poll, store, and section pattern to copy.
- Write the record format and the clear rules first, with tests, before any UI.
