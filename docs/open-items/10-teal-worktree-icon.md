# Teal worktree icon

Status: teal PNGs, .icns, .ico and Linux SVG are on main (PR 22); only `Herdr-worktree.car` is still red, blocked on Xcode

## Goal

Make the icon of a worktree build a shade of teal instead of red. A normal build keeps the cream and charcoal icon.

## Decisions already made

- The red icon is the deliberate worktree-build marker: `app_icon.rs` embeds `herdr-worktree-1024.png` and `Herdr-worktree.icns` when `HERDR_BUILD_WORKTREE` is 1. Only that variant changes.
- The red comes from a colour matrix in `scripts/generate-icons.swift` (`worktreePNG`, `worktreeColor`). Change that matrix to a teal mapping and regenerate; do not hand-edit the binaries.
- The stock `/Applications/Herdr.app` icon is not part of this repo and is not touched.

## Open questions

- Whether the stable icon should change too. The operator's request named the red icon only.
- The exact teal. Pick it from a rendered comparison, not by guessing matrix values.

## Blockers

- `rsvg-convert` is installed (`brew install librsvg`, 2026-11-01).
- Full Xcode 26 or later is not installed (it also blocks the `herdr-updater` test in item 08), so `xcrun actool` fails and `generate-icons.swift` stops (it needs `actool` for `Herdr-worktree.car`). After installing it, run `sudo xcode-select -s /Applications/Xcode.app`.
- Without Xcode, the PNG, `.icns`, `.ico` and Linux SVG are regenerated in teal (done) but the `.car` stays red, and `scripts/check-icons.swift` may then fail on the mismatch. Until the `.car` is rebuilt, a macOS 26 Dock shows red for a fresh worktree build; the operator's installed `Herdr Dev.app` was patched by hand to use the teal `.icns` without the `.car`. A rebuild copies the repo's `.car`, which is still red.

## Where to start

- `scripts/generate-icons.swift`, then `just icons`; Windows icons come from `just icons-windows` (`scripts/generate-windows-icons.py`).
- Assets to regenerate: `assets/icons/herdr-worktree-1024.png`, `herdr-square-worktree-1024.png`, `Herdr-worktree.icns`, `.car`, `.ico`, and `herdr-linux-worktree.svg`.
- `crates/herdr-gpui/src/app_icon.rs` has a test that the stable and worktree icons differ; keep it passing.
- Verify in the Dock with a worktree build, and run `scripts/check-icons.swift`.
