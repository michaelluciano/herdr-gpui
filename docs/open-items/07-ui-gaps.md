# UI gaps against the mockup

Status: not started (tracked by a draft PR; no code yet)

## Goal

Close the remaining differences between the app and the design mockup (see `docs/mockups`).

## Present

The items the mockup shows that the app already has are not listed here.

## Partial

- The filter label says "spaces" rather than "Filter agents...".
- CPU and MEM are numbers plus a dotted trace, not filled bars.
- Usage meters lack the 5h and 7d labels and a hot state.

## Missing

- A unified agent list with task title, age, branch, and model.
- A pane header with a goal pill and branch and model chips.
- The "..." menu.

## Not verified

- Worktree group headers.
- The alert chip.

## Related items

- The fork has no Files panel, so file-type icons are blocked on one. A Files panel is new, unplanned work.
- The daemon does not set `version_check` or `manifest_check` to false, so update checks reach the network by default. This is a daemon configuration item owned outside this repository, listed here so it is not lost. Unverified here: neither key appears in this repository's code.

## Decisions already made

- None yet; each gap needs its own small plan.

## Open questions

- Which gaps matter first; the unified agent list is the largest.

## Where to start

- Compare a screenshot of the running app with `docs/mockups` and mark each line above verified or not.
