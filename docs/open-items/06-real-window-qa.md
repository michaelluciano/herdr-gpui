# Real-window QA

Status: needs verification

## Goal

Confirm in a real window the behavior that unit tests do not cover. These checks need a build of current main running as the only instance (see item 05 for why).

## Checks

- The git graph gutter is 96 px wide (`GUTTER_CAP` in `crates/herdr-gpui/src/graph_view/geometry.rs`).
- Clicking a handoff row opens it in a code tab.
- Clicking a stopped Mac launcher row shows "starting", starts the model, and opens its web UI.
- Clicking a running launcher row only opens the UI.
- A blocked (gray) launcher row flashes the name of the model holding the slot and starts nothing.
- The remote terminal launcher row opens a tab, types its command, and runs it.
- A host with no workspace shows its flash. A click while another tab is opening shows its flash.
- Handoffs panel: the title folds; `picked up N` toggles the history; rows show `MM-DD HH:MM`, or `MM-DD` when no time is known.

## Not covered by any automated test

The terminal launcher's connection path: the tab-create request, reply handling, typing, and navigation. It needs a live connection to check.

## Decisions already made

- None beyond the checks above; record each result next to its line when done.

## Open questions

- Whether the remote terminal path should get a fake-daemon test so this check can be automated.

## Where to start

- Build current main, make sure no other instance is running, then walk the list.
- Launcher code: `crates/herdr-gpui/src/llm_launch.rs`, `llm_launch/click.rs`, `sidebar/llm_rows.rs`. Handoffs: `crates/herdr-gpui/src/sidebar/handoffs.rs`.
