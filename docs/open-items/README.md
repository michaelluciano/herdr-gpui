# Open items

This folder lists the work that is planned or unverified in this fork, one file per item, in priority order. Each item file states the goal, the decisions already made, the open questions, and where to start in the code. Nothing here is a commitment to a date; an item leaves the list when its pull request merges or when the check it describes has been done in a real window.

Last reviewed 2026-11-08. Status comes from the branches and tests named in each file; where a pull request is not named, none is open yet. Every feature branch from the 2026-11-07 list is merged into main; what is left is real-window checking, items 03, 04 and 07, and the decisions at the end of this file.

| Item | File | Status |
| --- | --- | --- |
| GitHub account chip above the sidebar footer | [02-github-account-chip.md](02-github-account-chip.md) | merged (PR 23); amber account color; needs a real-window check |
| Session self-naming | [03-session-self-naming.md](03-session-self-naming.md) | on hold; mechanism findings recorded |
| Needs-run command and "Needs you" section | [04-needs-run-and-needs-you.md](04-needs-run-and-needs-you.md) | not started |
| Control socket never retries its bind | [05-control-socket-retry.md](05-control-socket-retry.md) | merged (PR 33); needs a real-window check |
| Real-window QA of recent features | [06-real-window-qa.md](06-real-window-qa.md) | needs verification at a desktop; the list now covers the 2026-11-08 merges |
| Gaps against the UI mockup | [07-ui-gaps.md](07-ui-gaps.md) | not started |
| Cleanups and known nits | [08-cleanups.md](08-cleanups.md) | partly done in PR 31; updater test blocked on the toolchain; `lint-linux` and `lint-windows` never run on main |
| Teal worktree icon | [10-teal-worktree-icon.md](10-teal-worktree-icon.md) | assets merged (PR 22); only `Herdr-worktree.car` is still red, blocked on Xcode |
| Clear past handoffs that are not locked | [11-clear-unlocked-handoffs.md](11-clear-unlocked-handoffs.md) | future; padlock and dismissal exist, the bulk action and shared state do not |

## Blocked on something outside the code

- Full Xcode 26 or later is not installed. It blocks the `.car` in item 10 and the `herdr-updater` test in item 08: the Command Line Tools default SDK (MacOSX27.0) breaks the linker on this Mac. Until then, builds here need `SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk`.
- Item 06 needs a person at a desktop; no automated check covers it.

## Open decisions

- Trailers in history: main commit `8fe118d5` (the squash of PR 23) and the commits of the older feature branches carry `Co-Authored-By` trailers, which `CLAUDE.md` forbids. Rewriting main needs an explicit go-ahead; until then they stay.
- PR 21 (`chore/fix-check-sorted`) repeats the sort fix that reached main in PR 31. Close it once confirmed.
- PRs 7 to 13 and 16 are item documents written before the 2026-11-07 refresh (PR 32). Merging them would overwrite the refreshed statuses; close them or rebase them by hand.
- Model letter and context size on agent rows (PR 29) need the `transcript` key in each pane's activity state file. The Claude activity hook now writes it (2026-11-08); panes pick it up on their next hook event. Check it in a real window (item 06).
- Item 03 stays on hold.
- Item 11 (future): clear every past handoff that is not locked, with locks and clears shared across machines. Where the shared state lives is open.

## Merged on 2026-11-08, not yet checked in a real window

Account chip (PR 23), control socket rebind (PR 33), handoffs panel refresh within a second (PR 35), session lock (PR 27), agent drag (PR 28), model letter and context size (PR 29), divider floor (PR 25), ports scroll (PR 24), teal accent (PR 30), teal worktree strip and icons (PRs 26 and 22). The checks are listed in item 06. `just lint-linux` and `just lint-windows` have not been run on main.

## Done recently

- Feature branches merged into main on 2026-11-08: PRs 22 to 30, 33 and 35.
- `just ci` runs on main again (PR 31).
- Copy session id from the right-click menu (PR 6; this was item 01).
- Handoff row click copies its path, and right-click opens it in a code tab (PRs 14 and 15).
- Handoffs show the newest per project, read from `LATEST.md` and `history/` (PRs 18 and 20).
- Git graph tab and Handoffs section (PR 2).
- LLM launcher rows (PR 3).
- Handoffs panel fold, history, and timestamps (PR 4).
