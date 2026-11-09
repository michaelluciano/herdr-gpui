# Control socket never retries its bind

Status: fixed on `fix/control-socket-retry` (PR 33). A real-window check is still to do.

## Problem

The control socket (`control.sock`) is bound by the first app instance only. A second instance logs "Another Herdr UI already serves ..." (the error is in `crates/herdr-control-protocol/src/error.rs`) and never listens. It also does not retry when the first instance quits and releases the socket. The result is that `herdr-gpui graph open` and the other control commands fail with "control socket unreachable" while the app is running.

## Decisions already made

- Proposal: the surviving instance retries the bind periodically, or when the socket goes away, so it takes over once the first instance is gone.
- The fix needs a test.

## Open questions

- Retry on a timer or watch the socket path; a timer is simpler.
- How to avoid two instances racing to bind at the same moment (the existing stale-socket handling in `install` must still decide a winner).
- Retry interval and whether to log each failed attempt.

## What was done

- `install` in `crates/herdr-gpui/src/control/server.rs` now retries the bind every two seconds (`REBIND_EVERY`) while a live server holds the socket, off the UI thread, and starts serving on the first success. Only the instance that holds the socket removes it on quit. Any other bind error still stops with the existing warning.
- Test: `control/server/tests/rebind.rs` checks that a second install does not take a live socket and takes it once the first is released. It is Unix-only; the Windows named pipe path is not tested. All 182 `control::` tests, `cargo fmt` and clippy pass.
- Still open from the questions below: the retry is a timer, not a watch; a failed attempt is logged once, not per attempt.

## Workaround

Run a single instance of the app.

## Where to start

- `crates/herdr-gpui/src/control/server.rs`, function `install` (line 317 at the time of writing).
- Add a test with two installs against one socket path: the second binds after the first is dropped.
