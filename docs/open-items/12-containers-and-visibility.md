# Containers and visibility to them

Status: future item, not started; recorded 2026-11-08 at the operator's request

## Goal

Bring containers into the app and show them: which containers exist, which workspaces, panes and agents run inside them, and what state they are in, so that work inside a container is as visible in the sidebar as work on a host or cloud machine.

## What exists today

- The app shows machines (this computer's daemon, SSH hosts, cloud devices) and the workspaces on them. It has no notion of a container as a place work runs.
- `crates/herdr-remote` covers machines beyond the local daemon and `crates/herdr-client/src/wsl.rs` covers WSL on Windows. Neither is a container runtime.
- `docs/sandbox.md` mentions containers; read it before designing, to see whether it describes a sandbox for agents or something the app should list.

## Open questions

- Which runtimes count: Docker, Podman, Apple's container tool, dev containers, or all of them.
- What "visibility" means first: a read-only list with state, the workspaces and agents inside each container, logs, or resource use.
- Whether a container is a new kind of machine next to SSH hosts and cloud devices, or a child of the host that runs it.
- Whether the app starts, stops or enters containers, or only shows them. `CLAUDE.md` says destructive or outward actions go through the app's own confirmation dialog.
- Where the code lives: a new crate would need a concrete need (`CLAUDE.md`, Architecture), so check whether `herdr-remote` can hold it.

## Where to start

- Answer the questions above with the operator; do not build before then.
- Then add the read side first (list and state, off the UI thread, bounded), and a `herdr-gpui` control command for it with `--json`, per the Command Line rules in `CLAUDE.md`.
