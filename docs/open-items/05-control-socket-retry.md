# Control socket never retries its bind

Status: bug

## Problem

The control socket (`control.sock`) is bound by the first app instance only. A second instance logs "Another Herdr UI already serves ..." (the error is in `crates/herdr-control-protocol/src/error.rs`) and never listens. It also does not retry when the first instance quits and releases the socket. The result is that `herdr-gpui graph open` and the other control commands fail with "control socket unreachable" while the app is running.

## Decisions already made

- Proposal: the surviving instance retries the bind periodically, or when the socket goes away, so it takes over once the first instance is gone.
- The fix needs a test.

## Open questions

- Retry on a timer or watch the socket path; a timer is simpler.
- How to avoid two instances racing to bind at the same moment (the existing stale-socket handling in `install` must still decide a winner).
- Retry interval and whether to log each failed attempt.

## Workaround

Run a single instance of the app.

## Where to start

- `crates/herdr-gpui/src/control/server.rs`, function `install` (line 317 at the time of writing).
- Add a test with two installs against one socket path: the second binds after the first is dropped.
