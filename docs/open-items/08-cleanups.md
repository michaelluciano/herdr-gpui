# Cleanups and known nits

Status: not started

## Goal

Collect small known issues so they are fixed deliberately, each as its own scoped change.

## Items

- Git graph: the file list is capped at 200 rows with a "+N more" line (`MAX_FILE_ROWS` in `crates/herdr-gpui/src/graph_view/render/files.rs`). Every graph tab of the focused workspace reloads every 5 seconds (`GRAPH_REFRESH` in `graph_view/load.rs`). Graphs work on local checkouts only and are read-only.
- `crates/herdr-gpui/src/llm_launch.rs` keeps `#![allow(unused_imports)]` because some re-exports are used only by tests.
- The test-only `run_llm_plan` only counts clicks, so the real click path is covered by clippy and by the `run_llm_plan_with` tests only (`llm_launch/click.rs`, `sidebar/llm_rows.rs`).
- `just` is not installed on the Mac, so `just ci`, lint-linux, and lint-windows have not been run on recent work.
- One existing test fails when run alone and is unrelated to recent work: `install::tests::replacement::real_executable_archive_installs_relaunches_and_rolls_back` in `crates/herdr-updater` ("Update validation command failed").

## Decisions already made

- Refactors are separate, scoped changes, never drive-bys.

## Open questions

- Whether the 5 second reload should back off for tabs that are not visible.
- Whether the failing updater test depends on the environment or is a real regression.

## Where to start

- Install `just` and run `just ci` first; it may surface more than is listed here.
