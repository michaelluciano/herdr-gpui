use super::*;
use crate::HerdrWindow;
use crate::sidebar::layout_tests::{SidebarFixture, fixture_window};
use gpui::{AppContext as _, Entity, VisualTestContext, px, size};
use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

#[test]
fn the_root_is_claude_private_handoffs_under_home() {
    let home = Path::new("/Users/someone");
    assert_eq!(
        root_under(home),
        home.join("claude-private").join("handoffs")
    );
}

/// A window and a counter of how often it notified.
fn counted(
    cx: &mut gpui::TestAppContext,
) -> (Entity<HerdrWindow>, Rc<Cell<usize>>, &mut VisualTestContext) {
    let notified = Rc::new(Cell::new(0));
    let seen = notified.clone();
    let (fixture, cx) = cx.add_window_view(move |window, cx| {
        let view = cx.new(|cx| fixture_window(window, cx));
        cx.observe(&view, move |_, _, cx| {
            seen.set(seen.get() + 1);
            cx.notify();
        })
        .detach();
        SidebarFixture(view)
    });
    cx.simulate_resize(size(px(800.), px(600.)));
    let view = cx.update(|_, cx| fixture.read(cx).0.clone());
    (view, notified, cx)
}

fn poll(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, now: Instant, root: &Root) {
    let root = root.path().to_owned();
    cx.update(|_, cx| view.update(cx, |view, cx| view.poll_handoffs_at(now, root, cx)));
    cx.run_until_parked();
}

fn ids(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Vec<String> {
    view.read_with(cx, |view, _| {
        view.handoffs.list.iter().map(|h| h.id.clone()).collect()
    })
}

#[gpui::test]
fn a_poll_reads_the_root_off_thread_and_fills_the_list(cx: &mut gpui::TestAppContext) {
    let root = Root::new("poll-fill");
    root.session("p", "a.md", open_header("a", "2026-10-30"));
    root.session("p", "b.md", open_header("b", "2026-10-31"));
    let (view, notified, cx) = counted(cx);
    assert!(ids(&view, cx).is_empty());
    let before = notified.get();
    poll(&view, cx, Instant::now(), &root);
    assert_eq!(ids(&view, cx), ["b", "a"]);
    assert!(notified.get() > before, "a changed list notifies");
}

#[gpui::test]
fn a_poll_that_finds_the_same_list_does_not_notify(cx: &mut gpui::TestAppContext) {
    let root = Root::new("poll-same");
    root.session("p", "a.md", open_header("a", "2026-10-30"));
    let (view, notified, cx) = counted(cx);
    let t0 = Instant::now();
    poll(&view, cx, t0, &root);
    assert_eq!(ids(&view, cx), ["a"], "the first poll fills the list");
    let settled = notified.get();
    poll(&view, cx, t0 + Duration::from_secs(6), &root);
    poll(&view, cx, t0 + Duration::from_secs(12), &root);
    assert_eq!(ids(&view, cx), ["a"]);
    assert_eq!(notified.get(), settled, "an unchanged list stays quiet");
}

#[gpui::test]
fn a_changed_directory_updates_the_list_after_five_seconds(cx: &mut gpui::TestAppContext) {
    let root = Root::new("poll-change");
    let first = root.session("p", "a.md", open_header("a", "2026-10-30"));
    let (view, notified, cx) = counted(cx);
    let t0 = Instant::now();
    poll(&view, cx, t0, &root);
    assert_eq!(ids(&view, cx), ["a"]);

    root.session("p", "b.md", open_header("b", "2026-10-31"));
    // Inside the interval nothing is read.
    poll(&view, cx, t0 + Duration::from_secs(2), &root);
    assert_eq!(ids(&view, cx), ["a"], "polled again before five seconds");

    let before = notified.get();
    poll(&view, cx, t0 + Duration::from_secs(6), &root);
    assert_eq!(ids(&view, cx), ["b", "a"]);
    assert!(notified.get() > before);

    // Claiming a file drops it at the next due poll.
    std::fs::write(&first, "---\nid: a\nstatus: claimed\n---\n").unwrap();
    poll(&view, cx, t0 + Duration::from_secs(12), &root);
    assert_eq!(ids(&view, cx), ["b"]);
}
