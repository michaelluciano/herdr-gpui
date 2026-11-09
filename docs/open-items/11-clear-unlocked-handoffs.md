# Clear past handoffs that are not locked

Status: not started; requested 2026-11-08

## Goal

Add one action to the Handoffs panel that clears every past handoff that is not locked, so finished and superseded handoffs stop piling up. Locked handoffs always stay.

## What exists today

- A handoff's lock is the padlock on its row. It is stored as the pin set, `handoff-pinned.json` in the app's state directory (`crates/herdr-gpui/src/handoffs/pinned.rs`), per machine.
- Clearing a row is a dismissal: the row's x adds its path to `handoff-dismissed.json` (`handoffs/dismissed.rs`). The handoff file is never touched, and "Restore cleared (N)" brings every dismissed row back.
- The all view's bar (`bulk_actions` in `crates/herdr-gpui/src/sidebar/handoffs.rs`) offers "Clear picked up (N)" and "Restore cleared (N)". `Handoffs::clear_picked_up` in `handoffs/poll.rs` dismisses only picked-up rows, and it does not look at the pin set. Superseded rows can only be cleared one at a time.

## Decisions already made

- The action clears past handoffs: the picked-up and superseded rows. A locked handoff is never cleared.
- Clearing means dismissal, so it is reversible with "Restore cleared" and no file is deleted.

## Open questions

- Whether it also clears active handoffs (the newest open one per project) that are not locked. "Past" says no; confirm.
- Whether it replaces "Clear picked up (N)" or sits beside it. One bar entry that covers both shelves is simpler.
- Whether a confirmation is needed. Dismissal is undoable, so none is the default.
- Whether pins and dismissals should be shared across machines. Today each machine keeps its own state files.

## Where to start

- Add a method beside `clear_picked_up` in `crates/herdr-gpui/src/handoffs/poll.rs` that dismisses every picked-up and superseded path not in `self.pinned`, with tests in a new topic file under `handoffs/tests/`.
- Wire it in `bulk_actions` in `crates/herdr-gpui/src/sidebar/handoffs.rs` and give the control a layout test.
- Saving the dismissal set already runs off the UI thread; keep it that way.
