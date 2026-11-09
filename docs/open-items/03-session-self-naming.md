# Session self-naming

Status: on hold (operator, 2026-11-07); nothing built

## Goal

Give each workspace a useful name automatically. The failing case today is a workspace labelled with only the default working-directory name, which tells the operator nothing when several sessions share a directory.

## Decisions already made

- Name precedence: the claimed handoff id or project, then the worktree or branch name, then the directory name.
- Never overwrite a name the operator typed. Automatic names are marked as automatic so they can be told apart.
- The topic token stays as the secondary line of the row.

## Open questions

- Mechanism A: a session-start hook that calls `herdr workspace rename`. This changes the real workspace name.
- Mechanism B: `herdr workspace report-metadata --source <id> --token name=... --ttl-ms ...`, which is display-only and expires. Unverified: whether this fork renders such tokens at all.
- Unverified: how a session looks up its own workspace id to pass to either command.
- How an automatic name is marked, and how the operator resets a name back to automatic.

## Findings, 2026-11-07

- Mechanism B does not fit this fork's default sidebar. The fork draws daemon `report-metadata` tokens only in rows built from a configured sidebar layout (`crates/herdr-gpui/src/sidebar/row/configured.rs`, `plugin_values.rs`); the native name and detail lines ignore them. A token also expires with its TTL. This is read from the code; it was not tried against a live daemon, because that would write onto the operator's real workspaces.
- Mechanism A, `herdr workspace rename`, works in every layout and persists. The CLI has no flag that marks a name as automatic, so "never overwrite a name the operator typed" needs a convention. Recommended: the hook renames only when the current label still equals the default directory name, and applies the precedence above.
- A running daemon is reachable from a session with `herdr workspace list` (workspace ids look like `w6K`).
- Recommendation if work resumes: mechanism A with that rule. Start with the precedence function and its tests, which touch no daemon.

## Where to start

- Check how the sidebar picks a workspace label and secondary line (`crates/herdr-gpui/src/sidebar/`).
- Try mechanism B by hand against a running daemon to see whether the token appears.
- Decide on one mechanism before writing code; write a test for the precedence order as a pure function.
