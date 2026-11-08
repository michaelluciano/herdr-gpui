# Open items

This folder lists the work that is planned or unverified in this fork, one file per item, in priority order. Each item file states the goal, the decisions already made, the open questions, and where to start in the code. Nothing here is a commitment to a date; an item leaves the list when its pull request merges or when the check it describes has been done in a real window.

Last reviewed 2026-11-07. Status comes from the branches and tests named in each file; where a pull request is not named, none is open yet.

| Item | File | Status |
| --- | --- | --- |
| GitHub account chip above the sidebar footer | [02-github-account-chip.md](02-github-account-chip.md) | built on `feat/gh-account-chip`, tests green; needs PR and a real-window check |
| Session self-naming | [03-session-self-naming.md](03-session-self-naming.md) | on hold; mechanism findings recorded |
| Needs-run command and "Needs you" section | [04-needs-run-and-needs-you.md](04-needs-run-and-needs-you.md) | not started |
| Control socket never retries its bind | [05-control-socket-retry.md](05-control-socket-retry.md) | fixed on `fix/control-socket-retry`; PR waits on PR 31 |
| Real-window QA of recent features | [06-real-window-qa.md](06-real-window-qa.md) | needs verification at a desktop |
| Gaps against the UI mockup | [07-ui-gaps.md](07-ui-gaps.md) | not started |
| Cleanups and known nits | [08-cleanups.md](08-cleanups.md) | partly done in PR 31; updater test blocked on the toolchain |
| Teal worktree icon | [10-teal-worktree-icon.md](10-teal-worktree-icon.md) | all assets teal except `Herdr-worktree.car`, blocked on Xcode |

## Blocked on something outside the code

- Full Xcode 26 or later is not installed. It blocks the `.car` in item 10 and the `herdr-updater` test in item 08: the Command Line Tools default SDK (MacOSX27.0) breaks the linker on this Mac. Until then, builds here need `SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk`.
- Item 06 needs a person at a desktop; no automated check covers it.

## Built, not listed above, waiting on review or a pull request

These branches have commits that are not in main. They were found on 2026-11-07 and not reviewed beyond that.

- `feat/session-lock`: lock the agent panel's order.
- `feat/teal-accent`: default accent from lilac to teal.
- `feat/teal-worktree-icon`: ships the teal icon assets on main (item 10).
- `feat/agent-drag`, `feat/sidebar-divider-floor`, `feat/session-stats`, `feat/status-glyphs`, `feat/status-bar-ports-scroll`, `feat/handoff-latest-sidebar`, `chore/fix-check-sorted` (its commit is also in PR 31).

## Done recently

- Copy session id from the right-click menu (PR 6; this was item 01).
- Handoff row click copies its path, and right-click opens it in a code tab (PRs 14 and 15).
- Handoffs show the newest per project, read from `LATEST.md` and `history/` (PRs 18 and 20).
- Git graph tab and Handoffs section (PR 2).
- LLM launcher rows (PR 3).
- Handoffs panel fold, history, and timestamps (PR 4).
