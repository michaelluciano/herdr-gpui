# Nemotron launcher is not accessible

Status: parked (2026-11-01)

## Problem

The "nemotron" LLM launcher row cannot be reached from Herdr UI. What "not accessible" means has not been pinned down yet: the row may be missing from the sidebar, the launch may fail, or the model may not answer once started.

## Decisions already made

- Parked behind the status-bar usage change, terminal visibility, and the Handoffs slim-down.

## Open questions

- What fails: the row not showing, the launch, the host connection, or the model itself.
- Which host the launcher points at, and whether `ollama` and the `nemotron-3-nano:4b` model are installed there. On macluci there is no `[[llm_launchers]]` entry in `~/.config/herdr/config-gpui.toml` and no `ollama` on the path, so the config in use is on another machine.

## Where to start

- The launcher entry in the user's `config-gpui.toml` (`[[llm_launchers]]`, `label = "nemotron"`). The example config shows the shape: `kind = "terminal"`, `run = "ollama run nemotron-3-nano:4b"`.
- `crates/herdr-gpui/src/llm_launch.rs` and `crates/herdr-gpui/src/llm_launch/`, which plan and run a launch.
- `crates/herdr-gpui/src/sidebar/layout_tests/llm_rows.rs` for how the rows are drawn.
