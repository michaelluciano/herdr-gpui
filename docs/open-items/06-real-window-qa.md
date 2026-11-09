# Real-window QA

Status: needs verification at a desktop; no check below has been run since 2026-11-01. The 2026-11-08 checks at the end of the list are new.

## Goal

Confirm in a real window the behavior that unit tests do not cover. These checks need a build of current main running as the only instance (see item 05 for why).

## Checks

- The git graph gutter is 96 px wide (`GUTTER_CAP` in `crates/herdr-gpui/src/graph_view/geometry.rs`).
- Clicking a handoff row copies its absolute path to the clipboard and flashes it. Right-clicking a handoff row (active or picked up/superseded) opens its `.md` file in a code tab.
- Clicking a stopped Mac launcher row shows "starting", starts the model, and opens its web UI.
- Clicking a running launcher row only opens the UI.
- A blocked (gray) launcher row flashes the name of the model holding the slot and starts nothing.
- The remote terminal launcher row opens a tab, types its command, and runs it.
- A host with no workspace shows its flash. A click while another tab is opening shows its flash.
- Handoffs panel: the title folds; `picked up N` toggles the history; rows show `MM-DD HH:MM`, or `MM-DD` when no time is known.
- A new handoff file shows in the panel within about a second, and a deleted one leaves within about a second (PR 35).
- The GitHub chip shows the signed-in account in amber; a click switches to the next account; a repository owned by another account keeps the amber color and a semibold label (PR 23).
- Quit one Herdr UI while a second runs: the second takes over the control socket within about two seconds (PR 33).
- The session lock keeps the agent panel's order until unlocked (PR 27); dragging an agent row reorders the panel, and a drop on its own place changes nothing (PR 28).
- A Claude agent row shows its model letter beside the icon and its context size at the far right, in every row layout (PR 29). This needs a pane whose activity state file has a `transcript` key.
- The spaces/agents divider cannot be dragged below the workspace list (PR 25); the status bar ports scroll sideways instead of covering usage (PR 24).
- The default accent is teal (PR 30); a worktree build shows the teal strip and icon (PRs 26 and 22).

## Not covered by any automated test

The terminal launcher's connection path: the tab-create request, reply handling, typing, and navigation. It needs a live connection to check.

## Decisions already made

- None beyond the checks above; record each result next to its line when done.

## Open questions

- Whether the remote terminal path should get a fake-daemon test so this check can be automated.

## Where to start

- Build current main, make sure no other instance is running, then walk the list.
- Launcher code: `crates/herdr-gpui/src/llm_launch.rs`, `llm_launch/click.rs`, `sidebar/llm_rows.rs`. Handoffs: `crates/herdr-gpui/src/sidebar/handoffs.rs`.
