use super::poll::{counted, ids, poll};
use super::*;
use std::time::{Duration, Instant};

#[gpui::test]
fn a_new_handoff_shows_within_a_second_or_two(cx: &mut gpui::TestAppContext) {
    let root = Root::new("fast-new");
    root.session("p", "a.md", open_header("a", "2026-10-30"));
    let (view, _notified, cx) = counted(cx);
    let t0 = Instant::now();
    poll(&view, cx, t0, &root);
    assert_eq!(ids(&view, cx), ["a"]);

    root.session("p", "b.md", open_header("b", "2026-10-31"));
    poll(&view, cx, t0 + Duration::from_millis(1500), &root);
    assert_eq!(
        ids(&view, cx),
        ["b"],
        "the newer handoff of a project leads"
    );
}

#[gpui::test]
fn nothing_is_read_again_while_the_root_is_unchanged(cx: &mut gpui::TestAppContext) {
    let root = Root::new("fast-quiet");
    root.session("p", "a.md", open_header("a", "2026-10-30"));
    let (view, notified, cx) = counted(cx);
    let t0 = Instant::now();
    poll(&view, cx, t0, &root);
    let settled = notified.get();
    for second in 1..5 {
        poll(&view, cx, t0 + Duration::from_secs(second), &root);
    }
    assert_eq!(
        view.read_with(cx, |view, _| view.handoffs.last_poll),
        Some(t0),
        "an unchanged root is not listed before the five second read"
    );
    assert_eq!(notified.get(), settled);
}

#[gpui::test]
fn the_five_second_read_still_runs_when_nothing_changed(cx: &mut gpui::TestAppContext) {
    let root = Root::new("fast-floor");
    root.session("p", "a.md", open_header("a", "2026-10-30"));
    let (view, _notified, cx) = counted(cx);
    let t0 = Instant::now();
    poll(&view, cx, t0, &root);
    let later = t0 + Duration::from_secs(6);
    poll(&view, cx, later, &root);
    assert_eq!(
        view.read_with(cx, |view, _| view.handoffs.last_poll),
        Some(later)
    );
}

#[gpui::test]
fn a_removed_handoff_leaves_within_a_second_or_two(cx: &mut gpui::TestAppContext) {
    let root = Root::new("fast-gone");
    root.session("p", "a.md", open_header("a", "2026-10-30"));
    let newer = root.session("p", "b.md", open_header("b", "2026-10-31"));
    let (view, _notified, cx) = counted(cx);
    let t0 = Instant::now();
    poll(&view, cx, t0, &root);
    assert_eq!(ids(&view, cx), ["b"]);

    std::fs::remove_file(newer).unwrap();
    poll(&view, cx, t0 + Duration::from_millis(1500), &root);
    assert_eq!(ids(&view, cx), ["a"], "the older handoff returns");
}
