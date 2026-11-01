# Session self-naming

Status: not started

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

## Where to start

- Check how the sidebar picks a workspace label and secondary line (`crates/herdr-gpui/src/sidebar/`).
- Try mechanism B by hand against a running daemon to see whether the token appears.
- Decide on one mechanism before writing code; write a test for the precedence order as a pure function.
