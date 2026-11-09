# Clear past handoffs that are not locked

Status: future item, not started; answers recorded 2026-11-08, build on request

## Goal

Add one action to the Handoffs panel that clears every past handoff that is not locked, so finished and superseded handoffs stop piling up. Locked handoffs always stay.

## What exists today

- A handoff's lock is the padlock on its row. It is stored as the pin set, `handoff-pinned.json` in the app's state directory (`crates/herdr-gpui/src/handoffs/pinned.rs`), per machine.
- Clearing a row is a dismissal: the row's x adds its path to `handoff-dismissed.json` (`handoffs/dismissed.rs`). The handoff file is never touched, and "Restore cleared (N)" brings every dismissed row back.
- The all view's bar (`bulk_actions` in `crates/herdr-gpui/src/sidebar/handoffs.rs`) offers "Clear picked up (N)" and "Restore cleared (N)". It only shows in the all view while something is picked up, which is why it is easy to miss. `Handoffs::clear_picked_up` in `handoffs/poll.rs` dismisses only picked-up rows, and it does not look at the pin set. Superseded rows can only be cleared one at a time.

## Decisions already made

- The action clears past handoffs only: the picked-up and superseded rows. Active handoffs (the newest open one per project) are not cleared, and a locked handoff is never cleared.
- Clearing means dismissal, so it is reversible with "Restore cleared" and no file is deleted.
- No confirmation.
- Locks and clears are shared: every machine and window that reads the same handoffs shows the same locked and cleared rows. Today each machine keeps its own state files, so this changes where they are stored.
- The new action is one bar entry for both shelves, replacing "Clear picked up (N)", and it shows whenever there is a past row to clear (assumed; the answer did not say).

## Open questions

- Where the shared state lives and how it reaches other machines. The handoffs root is `~/claude-private/handoffs` on each machine (`root_under` in `handoffs/poll.rs`), so the natural place is a small file inside that root, read and written like the handoff files. That only reaches another machine if the root itself is shared or synced; check how it is today before choosing.
- How the existing per-machine files (`handoff-pinned.json`, `handoff-dismissed.json`) move to the shared place without losing anyone's locks, and what happens when two machines change the set at once.
- Whether Claude sessions that read the handoff root need to ignore the new file.

## Where to start

- Do not start before the shared-state question is answered. Then add a method beside `clear_picked_up` in `crates/herdr-gpui/src/handoffs/poll.rs` that dismisses every picked-up and superseded path not in `self.pinned`, with tests in a new topic file under `handoffs/tests/`.
- Wire it in `bulk_actions` in `crates/herdr-gpui/src/sidebar/handoffs.rs` and give the control a layout test.
- Saving the dismissal set already runs off the UI thread; keep it that way.
