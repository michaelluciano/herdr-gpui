# Text selection and copy in code tabs

Status: not started (tracked by a draft PR; no code yet)

## Goal

Let the operator highlight text in a code tab, including handoff `.md` files, and copy it with Cmd+C so it can be pasted into a session.

## Decisions already made

- Applies to every code tab, since handoffs open in the same view.
- Plain drag-to-select plus Cmd+C is the minimum. Double-click for a word and triple-click for a line were offered and are not yet decided.

## Open questions

- Whether double-click selects a word and triple-click a line.
- How selection interacts with the outline panel, syntax colouring, and the line the tab opened at.

## Where to start

- The code tab is a read-only view the app draws itself: `crates/herdr-gpui/src/code_view.rs` and `code_view/render.rs`. It has no selection today.
- Reuse the app's clipboard and flash pattern (`copy_session_id` in `crates/herdr-gpui/src/session_copy.rs`).
- Hit testing, painting, and any selection geometry must share one set of text metrics (see the GPUI rules in AGENTS.md).
- Test selection ranges and the copied text as pure logic, then the click and drag path headlessly.
