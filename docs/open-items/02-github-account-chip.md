# GitHub account chip

Status: not started

## Goal

Show which GitHub account the `gh` command line tool is currently using, and let the operator switch it, so a push or pull request is not made from the wrong account.

## Decisions already made

- The chip is its own row directly above the sidebar footer (the "All Devices" bar).
- It shows the active `gh` CLI account.
- Switching runs `gh auth switch -u <name>` through `herdr_client::new_command` (`crates/herdr-client/src/process.rs`) on the background executor, never on the UI thread.
- The chip turns amber when the focused workspace's repository owner differs from the active account.
- It never switches accounts by itself; only an explicit operator click does.
- Local host only at first.
- It is separate from the app's own GitHub sign-in (`crates/herdr-github`), which is used for pull request badges. The two must not share state or labels.

## Open questions

- How to read the focused workspace's repository owner reliably (remote URL parsing, and what to do with several remotes).
- How to list the accounts `gh` knows about (parse `gh auth status` or use its JSON output) and how often to refresh.
- What the chip shows when `gh` is missing or signed out.

## Where to start

- The sidebar footer code under `crates/herdr-gpui/src/sidebar/` (search for the "All Devices" device footer).
- `crates/herdr-client/src/process.rs` for `new_command`.
- Test the owner-mismatch decision as a pure function with fixtures.
