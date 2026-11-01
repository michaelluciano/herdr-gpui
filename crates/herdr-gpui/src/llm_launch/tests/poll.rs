use super::*;
use crate::HerdrWindow;
use crate::config::llm_launchers::{Kind, Launcher};
use crate::sidebar::layout_tests::{SidebarFixture, fixture_window};
use gpui::{AppContext as _, Entity, VisualTestContext, px, size};
use std::{
    cell::Cell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

fn rows(count: usize) -> Vec<Launcher> {
    (0..count)
        .map(|i| Launcher {
            label: format!("m{i}"),
            host: "Local".to_owned(),
            group: Some("g".to_owned()),
            kind: Kind::Llama {
                port: 8081 + i as u16,
                start: "start".to_owned(),
                url: None,
            },
        })
        .collect()
}

fn counted(
    cx: &mut gpui::TestAppContext,
    launchers: Vec<Launcher>,
) -> (Entity<HerdrWindow>, Rc<Cell<usize>>, &mut VisualTestContext) {
    let notified = Rc::new(Cell::new(0));
    let seen = notified.clone();
    let (fixture, cx) = cx.add_window_view(move |window, cx| {
        let view = cx.new(|cx| fixture_window(window, cx));
        view.update(cx, |view, _| view.config.llm_launchers = launchers);
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

fn start(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    now: Instant,
    observe: fn(&[Launcher]) -> Observed,
) {
    cx.update(|_, cx| view.update(cx, |view, cx| view.poll_llm_launchers_at(now, observe, cx)));
}

fn poll(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    now: Instant,
    observe: fn(&[Launcher]) -> Observed,
) {
    start(view, cx, now, observe);
    cx.run_until_parked();
}

fn seen(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Observed {
    view.read_with(cx, |view, _| view.llm_launch.observed.clone())
}

fn ready(rows: &[Launcher]) -> Observed {
    Observed {
        health: vec![Some(Health::Ready); rows.len()],
        foreign: vec![],
    }
}

// Each test counts its own calls: tests run in parallel.
static EMPTY_CALLS: AtomicUsize = AtomicUsize::new(0);
fn count_empty(rows: &[Launcher]) -> Observed {
    EMPTY_CALLS.fetch_add(1, Ordering::SeqCst);
    ready(rows)
}

#[gpui::test]
fn no_rows_means_no_look_and_no_notify(cx: &mut gpui::TestAppContext) {
    let (view, notified, cx) = counted(cx, Vec::new());
    let before = notified.get();
    poll(&view, cx, Instant::now(), count_empty);
    assert_eq!(EMPTY_CALLS.load(Ordering::SeqCst), 0);
    assert_eq!(notified.get(), before);
    assert_eq!(seen(&view, cx), Observed::default());
    assert!(view.read_with(cx, |view, _| view.llm_launch.last_poll.is_none()));
}

#[gpui::test]
fn the_first_poll_fills_observed_through_the_injected_fn(cx: &mut gpui::TestAppContext) {
    let (view, notified, cx) = counted(cx, rows(2));
    let before = notified.get();
    poll(&view, cx, Instant::now(), ready);
    assert_eq!(seen(&view, cx), ready(&rows(2)));
    assert!(notified.get() > before, "a changed observation notifies");
}

static THROTTLE_CALLS: AtomicUsize = AtomicUsize::new(0);
fn count_throttle(rows: &[Launcher]) -> Observed {
    THROTTLE_CALLS.fetch_add(1, Ordering::SeqCst);
    ready(rows)
}

#[gpui::test]
fn a_poll_within_five_seconds_does_not_look_and_one_after_does(cx: &mut gpui::TestAppContext) {
    let (view, _, cx) = counted(cx, rows(1));
    let t0 = Instant::now();
    poll(&view, cx, t0, count_throttle);
    assert_eq!(THROTTLE_CALLS.load(Ordering::SeqCst), 1);
    poll(&view, cx, t0 + Duration::from_secs(4), count_throttle);
    assert_eq!(THROTTLE_CALLS.load(Ordering::SeqCst), 1, "inside 5 s");
    poll(&view, cx, t0 + Duration::from_secs(5), count_throttle);
    assert_eq!(THROTTLE_CALLS.load(Ordering::SeqCst), 2, "at 5 s");
}

fn down(rows: &[Launcher]) -> Observed {
    Observed {
        health: vec![Some(Health::Down); rows.len()],
        foreign: vec![],
    }
}

#[gpui::test]
fn only_a_changed_observation_notifies(cx: &mut gpui::TestAppContext) {
    let (view, notified, cx) = counted(cx, rows(1));
    let t0 = Instant::now();
    poll(&view, cx, t0, ready);
    assert_eq!(seen(&view, cx), ready(&rows(1)), "the first poll fills it");
    let settled = notified.get();
    poll(&view, cx, t0 + Duration::from_secs(6), ready);
    assert_eq!(notified.get(), settled, "the same result stays quiet");
    poll(&view, cx, t0 + Duration::from_secs(12), down);
    assert_eq!(seen(&view, cx), down(&rows(1)));
    assert!(notified.get() > settled, "a changed result notifies");
}

static GUARD_CALLS: AtomicUsize = AtomicUsize::new(0);
fn count_guard(rows: &[Launcher]) -> Observed {
    GUARD_CALLS.fetch_add(1, Ordering::SeqCst);
    ready(rows)
}

#[gpui::test]
fn the_loading_guard_prevents_a_second_concurrent_look(cx: &mut gpui::TestAppContext) {
    let (view, _, cx) = counted(cx, rows(1));
    let t0 = Instant::now();
    start(&view, cx, t0, count_guard);
    assert!(view.read_with(cx, |view, _| view.llm_launch.loading));
    // Due by the clock, but the first look has not come back.
    start(&view, cx, t0 + Duration::from_secs(10), count_guard);
    cx.run_until_parked();
    assert_eq!(GUARD_CALLS.load(Ordering::SeqCst), 1);
    assert!(!view.read_with(cx, |view, _| view.llm_launch.loading));
}

#[gpui::test]
fn a_result_that_arrives_after_the_rows_changed_is_still_stored(cx: &mut gpui::TestAppContext) {
    let (view, _, cx) = counted(cx, rows(3));
    start(&view, cx, Instant::now(), ready);
    view.update(cx, |view, _| view.config.llm_launchers = rows(1));
    cx.run_until_parked();
    assert_eq!(seen(&view, cx), ready(&rows(3)));
    let states = view.read_with(cx, |view, _| {
        states(&view.config.llm_launchers, &view.llm_launch.observed)
    });
    assert_eq!(states, [RowState::Running]);
}
