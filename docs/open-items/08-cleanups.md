# Cleanups and known nits

Status: not started

## Goal

Collect small known issues so they are fixed deliberately, each as its own scoped change.

## Items

- Git graph: the file list is capped at 200 rows with a "+N more" line (`MAX_FILE_ROWS` in `crates/herdr-gpui/src/graph_view/render/files.rs`). Every graph tab of the focused workspace reloads every 5 seconds (`GRAPH_REFRESH` in `graph_view/load.rs`). Graphs work on local checkouts only and are read-only.
- `crates/herdr-gpui/src/llm_launch.rs` keeps `#![allow(unused_imports)]` because some re-exports are used only by tests.
- The test-only `run_llm_plan` only counts clicks, so the real click path is covered by clippy and by the `run_llm_plan_with` tests only (`llm_launch/click.rs`, `sidebar/llm_rows.rs`).
- `just` is at `/opt/homebrew/bin/just` on the Mac (add it to PATH). `just ci` runs on macOS; `just lint-linux` and `just lint-windows` have not been run on recent work. `just ci` was red on main for two reasons now fixed in PR `fix/main-gates`: unsorted keep-sorted regions in `window.rs` and `window/construct.rs`, and `handoffs_edited` missing its `#[cfg(test)]` gate in `sidebar/layout_tests.rs`.
- `install::tests::replacement::real_executable_archive_installs_relaunches_and_rolls_back` in `crates/herdr-updater` fails on this Mac ("Update validation command failed"). Cause, verified 2026-11-07: the test builds its fixture with `/usr/bin/cc` under `env_clear()`, and the Command Line Tools default SDK (MacOSX27.0) has `.tbd` files the installed `ld` cannot parse ("tapi error: malformed file"). With `SDKROOT` set to the 26.5 SDK the same compile links. It is the machine, not a regression. Installing full Xcode (also needed for item 10) or repairing the Command Line Tools fixes it; other Cargo builds here need `SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk`.

## Decisions already made

- Refactors are separate, scoped changes, never drive-bys.

## Open questions

- Whether the 5 second reload should back off for tabs that are not visible.

## Where to start

- Run `just ci` with `SDKROOT` set as above; it is green except the updater test until Xcode is installed.
