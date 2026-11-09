# Clear handoffs that are not locked

Status: future item, not started; answers recorded 2026-11-08, build on request

## Goal

Give the Handoffs panel a way to clear every handoff that is not locked, so finished and superseded handoffs stop piling up in the list.

## Decisions already made

- The action clears past handoffs only: the picked-up and superseded rows. Active handoffs (the newest open one per project) are not cleared, and a locked handoff is never cleared.
- Clearing means dismissal, so it is reversible with "Restore cleared" and no file is deleted.
- No confirmation.
- Locks and clears are shared: every machine and window that reads the same handoffs shows the same locked and cleared rows. Today each machine keeps its own state files, so this changes where they are stored.
- "Clear picked up (N)" is not visible today because the bar only shows in the all view while something is picked up. The new action is one bar entry for both shelves, replacing "Clear picked up (N)", and it shows whenever there is a past row to clear (assumed; the answer did not say).

## Open questions

- Where the shared state lives and how it reaches other machines. The handoffs root is `~/claude-private/handoffs` on each machine (`root_under` in `handoffs/poll.rs`), so the natural place is a small file inside that root, read and written like the handoff files. That only reaches another machine if the root itself is shared or synced; check how it is today before choosing.
- How the existing per-machine files (`handoff-pinned.json`, `handoff-dismissed.json`) move to the shared place without losing anyone's locks, and what happens when two machines change the set at once.
- Whether Claude sessions that read the handoff root need to ignore the new file.

## Where to start

- Handoffs panel: `crates/herdr-gpui/src/sidebar/handoffs.rs`; the list and its kinds are in `crates/herdr-gpui/src/handoffs/list.rs`; the refresh is `handoffs/poll.rs` (a changed file shows within about a second, PR 35).
- Do the file work on the background executor, never on the UI thread.
