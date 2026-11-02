# LLM usage per host in the usage panel

Status: parked (2026-11-01)

## Goal

Next to the Local usage, the usage panel also shows the LLM usage of remoteluci and macluci. For each model on each host, over the past 24 hours, it shows:

- tokens in and tokens out
- average tokens per second
- run time

## Decisions already made

- Parked behind the status-bar usage change, terminal visibility, and the Handoffs slim-down.

## Open questions

- Where each host records these numbers. The options are an inference server's own logs or metrics (llama.cpp, ollama), a proxy that keeps a usage log (LiteLLM or similar), or a small collector this fork adds. The panel can only show what one of these keeps. At the time of writing, macluci has no `ollama` on the path.
- How the app reads them on a remote host: over Herdr's existing host connection, as the usage worker reads provider credentials on a host, or from an HTTP endpoint.
- Whether this is a usage provider in `crates/herdr-usage` (one report per host) or a new section in the panel.

## Where to start

- `crates/herdr-usage` (the providers, the registry, the read worker) and `docs/usage-providers.md`.
- The panel in `crates/herdr-gpui/src/usage/panel.rs` and `crates/herdr-gpui/src/usage/section.rs`.
- The host picker in the usage state (`Host::Local` and the remote hosts) in `crates/herdr-gpui/src/usage.rs`.
