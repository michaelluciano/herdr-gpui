use super::*;
use crate::HerdrWindow;
use crate::browser::{Location, Store, scope};
use crate::config::llm_launchers::{Kind, Launcher};
use crate::llm_launch::click::TERMINAL_RUNS;
use crate::sidebar::layout_tests::{fixture_window, snapshot};
use gpui::{Entity, TestAppContext, VisualTestContext};
use std::collections::{HashMap, HashSet};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};

fn llama(label: &str, host: &str, port: u16) -> Launcher {
    Launcher {
        label: label.into(),
        host: host.into(),
        group: Some("g".into()),
        kind: Kind::Llama {
            port,
            start: format!("start-{label}"),
            url: None,
        },
    }
}

/// Three llama rows in one group, then a terminal row.
fn rows() -> Vec<Launcher> {
    vec![
        llama("coder", "Local", 8081),
        llama("reason", "Local", 8082),
        llama("gemma", "Local", 8083),
        Launcher {
            label: "term".into(),
            host: "Local".into(),
            group: None,
            kind: Kind::Terminal { run: "true".into() },
        },
    ]
}

fn seen(health: [Option<Health>; 4], foreign: Vec<u16>) -> Observed {
    Observed {
        health: health.to_vec(),
        foreign,
    }
}

const DOWN: Option<Health> = Some(Health::Down);
const READY: Option<Health> = Some(Health::Ready);
const LOADING: Option<Health> = Some(Health::Loading);

fn set(indices: &[usize]) -> HashSet<usize> {
    indices.iter().copied().collect()
}

// ---- pure: states_with_starting and starting_rows ----

#[test]
fn a_starting_row_that_is_down_or_unseen_counts_as_starting() {
    let launchers = rows();
    let down = seen([DOWN, DOWN, DOWN, None], vec![]);
    let want = [
        RowState::Starting,
        RowState::Blocked {
            by: Blocker::Row(0),
        },
        RowState::Blocked {
            by: Blocker::Row(0),
        },
        RowState::Stopped,
    ];
    assert_eq!(states_with_starting(&launchers, &down, &set(&[0])), want);
    let unseen = seen([None, DOWN, DOWN, None], vec![]);
    assert_eq!(states_with_starting(&launchers, &unseen, &set(&[0])), want);
}

#[test]
fn a_starting_row_that_is_ready_stays_running_and_loading_stays_starting() {
    let launchers = rows();
    let ready = seen([READY, DOWN, DOWN, None], vec![]);
    assert_eq!(
        states_with_starting(&launchers, &ready, &set(&[0]))[0],
        RowState::Running
    );
    let loading = seen([LOADING, DOWN, DOWN, None], vec![]);
    assert_eq!(
        states_with_starting(&launchers, &loading, &set(&[0]))[0],
        RowState::Starting
    );
}

#[test]
fn without_a_starting_row_the_states_are_unchanged() {
    let launchers = rows();
    for observed in [
        seen([DOWN, DOWN, DOWN, None], vec![]),
        seen([READY, DOWN, DOWN, None], vec![]),
        seen([DOWN, DOWN, DOWN, None], vec![9000]),
    ] {
        assert_eq!(
            states_with_starting(&launchers, &observed, &HashSet::new()),
            states(&launchers, &observed)
        );
    }
}

#[test]
fn a_terminal_row_never_counts_as_starting() {
    let launchers = rows();
    let observed = seen([DOWN, DOWN, DOWN, None], vec![]);
    assert_eq!(
        states_with_starting(&launchers, &observed, &set(&[3]))[3],
        RowState::Stopped
    );
}

fn key(host: &str, port: u16) -> (String, u16) {
    (host.to_owned(), port)
}

#[test]
fn starting_rows_honours_the_ttl() {
    let launchers = rows();
    let t0 = Instant::now();
    let starting = HashMap::from([key("local", 8081)].map(|k| (k, t0)));
    let at = |minutes: u64| {
        starting_rows(
            &launchers,
            &starting,
            t0 + Duration::from_secs(60 * minutes),
        )
    };
    assert_eq!(at(0), set(&[0]));
    assert_eq!(at(19), set(&[0]));
    assert_eq!(at(21), HashSet::new());
}

#[test]
fn starting_rows_ignores_unknown_keys_and_other_hosts() {
    let launchers = rows();
    let t0 = Instant::now();
    let starting = HashMap::from(
        [
            key("local", 9999),
            key("remoteluci", 8081),
            key("local", 8082),
        ]
        .map(|k| (k, t0)),
    );
    assert_eq!(starting_rows(&launchers, &starting, t0), set(&[1]));
}

#[test]
fn starting_rows_canonicalises_local() {
    let t0 = Instant::now();
    let starting = HashMap::from([(key("local", 8081), t0)]);
    for host in ["Local", "local"] {
        let launchers = vec![llama("coder", host, 8081)];
        assert_eq!(
            starting_rows(&launchers, &starting, t0),
            set(&[0]),
            "{host}"
        );
    }
}

// ---- the window: run_llm_plan_with ----

/// Counting stubs for one test; tests run in parallel, so each owns a module.
macro_rules! stubs {
    ($name:ident, observe: $observe:expr, start: $start:expr) => {
        #[allow(dead_code)]
        mod $name {
            use super::*;
            pub(super) static OBSERVES: AtomicUsize = AtomicUsize::new(0);
            pub(super) static STARTS: Mutex<Vec<String>> = Mutex::new(Vec::new());
            fn observe(rows: &[Launcher]) -> Observed {
                OBSERVES.fetch_add(1, Ordering::SeqCst);
                let _ = rows;
                $observe
            }
            fn start(command: &str) -> Result<(), StartError> {
                STARTS.lock().unwrap().push(command.to_owned());
                $start
            }
            pub(super) fn effects() -> Effects {
                Effects { observe, start }
            }
            pub(super) fn observes() -> usize {
                OBSERVES.load(Ordering::SeqCst)
            }
            pub(super) fn starts() -> Vec<String> {
                STARTS.lock().unwrap().clone()
            }
        }
    };
}

fn window<'a>(
    cx: &'a mut TestAppContext,
    focused: Option<&str>,
    observed: Observed,
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext) {
    let focused = focused.map(str::to_owned);
    cx.add_window_view(move |window, cx| {
        let mut view = fixture_window(window, cx);
        let mut shown = snapshot(2);
        shown.focused_workspace_id = focused;
        view.live.snapshot = Some(Arc::new(shown));
        view.endpoints[0].live = view.live.clone();
        view.config.llm_launchers = rows();
        view.llm_launch.observed = observed;
        view
    })
}

fn run(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    index: usize,
    plan: Plan,
    effects: Effects,
) {
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.run_llm_plan_with(index, plan, effects, window, cx)
        })
    });
}

fn state(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, index: usize) -> RowState {
    view.read_with(cx, |view, _| view.llm_states(Instant::now())[index])
}

fn flash(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |view, _| {
        view.flash.as_ref().map(|(flash, _)| flash.text.to_string())
    })
}

fn pages(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Vec<String> {
    cx.update(|_, cx| {
        let scope = scope(&view.read(cx).endpoints[0]);
        Store::update(cx, |store| {
            store
                .in_workspace(&scope, "w0")
                .filter_map(|tab| match &tab.location {
                    Some(Location::Web { url }) => Some(url.as_str().to_owned()),
                    _ => None,
                })
                .collect()
        })
    })
}

fn starting_len(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> usize {
    view.read_with(cx, |view, _| view.llm_launch.starting.len())
}

stubs!(open, observe: Observed::default(), start: Ok(()));

#[gpui::test]
fn open_shows_the_page_in_the_focused_workspace_without_starting(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, Some("w0"), seen([READY, None, None, None], vec![]));
    run(&view, cx, 0, Plan::Open, open::effects());
    cx.run_until_parked();
    assert_eq!(pages(&view, cx), ["http://127.0.0.1:8081/"]);
    assert_eq!(open::observes(), 0);
    assert!(open::starts().is_empty());
    assert_eq!(starting_len(&view, cx), 0);
}

/// A window where the window's live state and the Local endpoint's own
/// state disagree about the focused workspace.
fn split_window<'a>(
    cx: &'a mut TestAppContext,
    selected: usize,
    window_focus: Option<&str>,
    endpoint_focus: Option<&str>,
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext) {
    let (window_focus, endpoint_focus) = (
        window_focus.map(str::to_owned),
        endpoint_focus.map(str::to_owned),
    );
    cx.add_window_view(move |window, cx| {
        let mut view = fixture_window(window, cx);
        let shown = |focus: Option<String>| {
            let mut shown = snapshot(2);
            shown.focused_workspace_id = focus;
            Some(Arc::new(shown))
        };
        view.live.snapshot = shown(window_focus);
        view.endpoints[0].live.snapshot = shown(endpoint_focus);
        view.endpoints.push(crate::endpoint::Endpoint::new(
            "ssh:test".into(),
            "Remote".into(),
            herdr_client::ConnectTarget::Ssh {
                target: "unused".into(),
                session: "default".into(),
            },
            true,
        ));
        view.selected_endpoint = selected;
        view.config.llm_launchers = rows();
        view.llm_launch.observed = seen([READY, None, None, None], vec![]);
        view
    })
}

stubs!(selected_local, observe: Observed::default(), start: Ok(()));

#[gpui::test]
fn open_with_local_selected_uses_the_window_live_state(cx: &mut TestAppContext) {
    let (view, cx) = split_window(cx, 0, Some("w0"), None);
    run(&view, cx, 0, Plan::Open, selected_local::effects());
    cx.run_until_parked();
    assert_eq!(pages(&view, cx), ["http://127.0.0.1:8081/"]);
    assert_eq!(flash(&view, cx), None);
}

stubs!(other_selected, observe: Observed::default(), start: Ok(()));

#[gpui::test]
fn open_with_another_endpoint_selected_uses_the_local_endpoints_state(cx: &mut TestAppContext) {
    let (view, cx) = split_window(cx, 1, None, Some("w0"));
    run(&view, cx, 0, Plan::Open, other_selected::effects());
    cx.run_until_parked();
    assert_eq!(pages(&view, cx), ["http://127.0.0.1:8081/"]);
    assert_eq!(flash(&view, cx), None);
}

stubs!(no_ws, observe: Observed::default(), start: Ok(()));

#[gpui::test]
fn open_without_a_workspace_flashes_and_opens_nothing(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, None, seen([READY, None, None, None], vec![]));
    run(&view, cx, 0, Plan::Open, no_ws::effects());
    cx.run_until_parked();
    assert_eq!(
        flash(&view, cx).as_deref(),
        Some("Open a workspace on this Mac first.")
    );
    assert!(pages(&view, cx).is_empty());
    assert!(no_ws::starts().is_empty());
}

stubs!(happy, observe: seen([DOWN, DOWN, DOWN, None], vec![]), start: Ok(()));

#[gpui::test]
fn start_then_open_marks_the_row_at_once_then_starts_and_opens(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, Some("w0"), seen([DOWN, DOWN, DOWN, None], vec![]));
    view.update(cx, |view, _| {
        view.llm_launch.last_poll = Some(Instant::now())
    });
    run(&view, cx, 0, Plan::StartThenOpen, happy::effects());
    // Before the background task runs.
    assert_eq!(state(&view, cx, 0), RowState::Starting);
    assert_eq!(
        state(&view, cx, 1),
        RowState::Blocked {
            by: Blocker::Row(0)
        }
    );
    assert_eq!(happy::observes(), 0);
    assert!(happy::starts().is_empty());
    cx.run_until_parked();
    assert_eq!(happy::observes(), 1);
    assert_eq!(happy::starts(), ["start-coder"]);
    assert_eq!(pages(&view, cx), ["http://127.0.0.1:8081/"]);
    assert_eq!(starting_len(&view, cx), 0);
    assert!(view.read_with(cx, |view, _| view.llm_launch.last_poll.is_none()));
}

stubs!(sibling, observe: seen([DOWN, READY, DOWN, None], vec![]), start: Ok(()));

#[gpui::test]
fn a_sibling_that_came_up_aborts_the_start_with_the_blocked_flash(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, Some("w0"), seen([DOWN, DOWN, DOWN, None], vec![]));
    run(&view, cx, 0, Plan::StartThenOpen, sibling::effects());
    cx.run_until_parked();
    assert_eq!(sibling::observes(), 1);
    assert!(sibling::starts().is_empty());
    assert_eq!(
        flash(&view, cx).as_deref(),
        Some("reason is running. Stop it first.")
    );
    assert_eq!(starting_len(&view, cx), 0);
    assert!(pages(&view, cx).is_empty());
}

stubs!(itself, observe: seen([READY, DOWN, DOWN, None], vec![]), start: Ok(()));

#[gpui::test]
fn a_row_that_is_already_running_is_opened_without_starting(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, Some("w0"), seen([DOWN, DOWN, DOWN, None], vec![]));
    run(&view, cx, 0, Plan::StartThenOpen, itself::effects());
    cx.run_until_parked();
    assert!(itself::starts().is_empty());
    assert_eq!(pages(&view, cx), ["http://127.0.0.1:8081/"]);
    assert_eq!(starting_len(&view, cx), 0);
}

stubs!(loading, observe: seen([LOADING, DOWN, DOWN, None], vec![]), start: Ok(()));

#[gpui::test]
fn a_row_that_is_still_loading_is_not_started_again(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, Some("w0"), seen([DOWN, DOWN, DOWN, None], vec![]));
    run(&view, cx, 0, Plan::StartThenOpen, loading::effects());
    cx.run_until_parked();
    assert!(loading::starts().is_empty());
    assert_eq!(flash(&view, cx).as_deref(), Some("coder is still loading."));
    assert!(pages(&view, cx).is_empty());
    assert_eq!(starting_len(&view, cx), 0);
}

stubs!(foreign, observe: seen([DOWN, DOWN, DOWN, None], vec![9000]), start: Ok(()));

#[gpui::test]
fn a_foreign_model_that_came_up_aborts_the_start(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, Some("w0"), seen([DOWN, DOWN, DOWN, None], vec![]));
    run(&view, cx, 0, Plan::StartThenOpen, foreign::effects());
    cx.run_until_parked();
    assert!(foreign::starts().is_empty());
    assert_eq!(
        flash(&view, cx).as_deref(),
        Some("Another model is running on port 9000. Stop it first.")
    );
}

stubs!(failed, observe: seen([DOWN, DOWN, DOWN, None], vec![]), start: Err(StartError::Failed(Some(1))));

#[gpui::test]
fn a_failed_start_flashes_the_exit_code_and_clears_the_row(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, Some("w0"), seen([DOWN, DOWN, DOWN, None], vec![]));
    run(&view, cx, 0, Plan::StartThenOpen, failed::effects());
    cx.run_until_parked();
    assert_eq!(failed::starts(), ["start-coder"]);
    assert_eq!(
        flash(&view, cx).as_deref(),
        Some("coder did not start (exit 1).")
    );
    assert_eq!(starting_len(&view, cx), 0);
    assert!(pages(&view, cx).is_empty());
}

stubs!(killed, observe: seen([DOWN, DOWN, DOWN, None], vec![]), start: Err(StartError::Failed(None)));

#[gpui::test]
fn a_killed_start_says_killed(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, Some("w0"), seen([DOWN, DOWN, DOWN, None], vec![]));
    run(&view, cx, 0, Plan::StartThenOpen, killed::effects());
    cx.run_until_parked();
    assert_eq!(
        flash(&view, cx).as_deref(),
        Some("coder did not start (killed).")
    );
}

stubs!(
    spawn,
    observe: seen([DOWN, DOWN, DOWN, None], vec![]),
    start: Err(StartError::Spawn(std::io::Error::other("boom")))
);

#[gpui::test]
fn a_start_that_cannot_spawn_flashes_the_error(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, Some("w0"), seen([DOWN, DOWN, DOWN, None], vec![]));
    run(&view, cx, 0, Plan::StartThenOpen, spawn::effects());
    cx.run_until_parked();
    assert_eq!(
        flash(&view, cx).as_deref(),
        Some("coder did not start: boom.")
    );
    assert_eq!(starting_len(&view, cx), 0);
}

stubs!(double, observe: seen([DOWN, DOWN, DOWN, None], vec![]), start: Ok(()));

#[gpui::test]
fn a_second_click_while_starting_does_nothing(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, Some("w0"), seen([DOWN, DOWN, DOWN, None], vec![]));
    run(&view, cx, 0, Plan::StartThenOpen, double::effects());
    run(&view, cx, 0, Plan::StartThenOpen, double::effects());
    cx.run_until_parked();
    assert_eq!(double::observes(), 1);
    assert_eq!(double::starts().len(), 1);
}

stubs!(terminal, observe: Observed::default(), start: Ok(()));

#[gpui::test]
fn a_terminal_row_goes_to_the_terminal_runner(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, Some("w0"), seen([DOWN, DOWN, DOWN, None], vec![]));
    let before = TERMINAL_RUNS.with(|runs| runs.get());
    run(&view, cx, 3, Plan::StartThenOpen, terminal::effects());
    cx.run_until_parked();
    assert_eq!(TERMINAL_RUNS.with(|runs| runs.get()), before + 1);
    assert_eq!(terminal::observes(), 0);
    assert!(terminal::starts().is_empty());
    assert_eq!(starting_len(&view, cx), 0);
}

stubs!(range, observe: Observed::default(), start: Ok(()));

#[gpui::test]
fn an_out_of_range_index_does_nothing(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, Some("w0"), seen([DOWN, DOWN, DOWN, None], vec![]));
    let before = TERMINAL_RUNS.with(|runs| runs.get());
    run(&view, cx, 99, Plan::StartThenOpen, range::effects());
    run(&view, cx, 99, Plan::Open, range::effects());
    cx.run_until_parked();
    assert_eq!(range::observes(), 0);
    assert!(range::starts().is_empty());
    assert_eq!(starting_len(&view, cx), 0);
    assert_eq!(flash(&view, cx), None);
    assert!(pages(&view, cx).is_empty());
    assert_eq!(TERMINAL_RUNS.with(|runs| runs.get()), before);
}
