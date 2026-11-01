use super::*;
use crate::HerdrWindow;
use crate::config::llm_launchers::{Kind, Launcher};
use crate::endpoint::Endpoint;
use crate::llm_launch::click::TERMINAL_RUNS;
use crate::sidebar::layout_tests::fixture_window;
use gpui::{Entity, TestAppContext, VisualTestContext};
use serde_json::json;
use std::time::{Duration, Instant};

// ---- pure ----

const ENDPOINTS: [(&str, &str); 3] = [
    ("local", "Local"),
    ("ssh:remoteluci", "remoteluci"),
    ("ssh:other", "remoteluci"),
];

#[test]
fn resolve_by_label_and_by_id() {
    assert_eq!(resolve_endpoint(&ENDPOINTS, "remoteluci"), Some(1));
    assert_eq!(resolve_endpoint(&ENDPOINTS, "ssh:other"), Some(2));
}

#[test]
fn resolve_treats_both_local_spellings_as_one() {
    assert_eq!(resolve_endpoint(&ENDPOINTS, "Local"), Some(0));
    assert_eq!(resolve_endpoint(&ENDPOINTS, "local"), Some(0));
}

#[test]
fn resolve_first_match_wins_and_unknown_is_none() {
    let twice = [("a", "x"), ("b", "x")];
    assert_eq!(resolve_endpoint(&twice, "x"), Some(0));
    assert_eq!(resolve_endpoint(&ENDPOINTS, "nowhere"), None);
    assert_eq!(resolve_endpoint(&[], "local"), None);
}

#[test]
fn pick_workspace_prefers_the_focused_one_when_listed() {
    assert_eq!(
        pick_workspace(Some("w1"), &["w0", "w1"]).as_deref(),
        Some("w1")
    );
}

#[test]
fn pick_workspace_falls_back_to_the_first() {
    assert_eq!(
        pick_workspace(Some("gone"), &["w0", "w1"]).as_deref(),
        Some("w0")
    );
    assert_eq!(pick_workspace(None, &["w0", "w1"]).as_deref(), Some("w0"));
    assert_eq!(pick_workspace(Some("w0"), &[]), None);
    assert_eq!(pick_workspace(None, &[]), None);
}

#[test]
fn create_params_are_exactly_workspace_and_focus() {
    assert_eq!(
        create_params("w3"),
        json!({"workspace_id": "w3", "focus": true})
    );
}

#[test]
fn the_gate_lets_a_pending_job_win_over_busy() {
    assert_eq!(click_gate(false, false), Gate::Go);
    assert_eq!(click_gate(true, false), Gate::Busy);
    assert_eq!(click_gate(true, true), Gate::Pending);
    assert_eq!(click_gate(false, true), Gate::Pending);
}

// ---- the window ----

const RUN: &str = "ollama run nemotron-3-nano:4b";
const LABEL: &str = "nemotron-3-nano:4b";

fn launcher() -> Launcher {
    Launcher {
        label: LABEL.into(),
        host: "remoteluci".into(),
        group: None,
        kind: Kind::Terminal { run: RUN.into() },
    }
}

/// A window with the terminal row and, when `with_host`, a never-dialing
/// "remoteluci" endpoint.
fn window(
    cx: &mut TestAppContext,
    with_host: bool,
) -> (Entity<HerdrWindow>, &mut VisualTestContext) {
    cx.add_window_view(move |window, cx| {
        let mut view = fixture_window(window, cx);
        if with_host {
            let mut host = Endpoint::new(
                "ssh:remoteluci".into(),
                "remoteluci".into(),
                herdr_client::ConnectTarget::Ssh {
                    target: "unused".into(),
                    session: "default".into(),
                },
                true,
            );
            host.never_connect();
            view.endpoints.push(host);
        }
        view.config.llm_launchers = vec![launcher()];
        view
    })
}

fn click(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, index: usize) {
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.run_terminal_launcher(index, window, cx))
    });
}

fn flash(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |view, _| {
        view.flash.as_ref().map(|(flash, _)| flash.text.to_string())
    })
}

fn job(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Option<(String, String)> {
    view.read_with(cx, |view, _| {
        view.llm_launch
            .terminal
            .as_ref()
            .map(|job| (job.endpoint.clone(), job.line.clone()))
    })
}

#[gpui::test]
fn a_host_that_is_not_set_up_is_said_so(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, false);
    click(&view, cx, 0);
    assert_eq!(
        flash(&view, cx).as_deref(),
        Some("nemotron-3-nano:4b: host remoteluci is not set up.")
    );
    assert_eq!(job(&view, cx), None);
}

#[gpui::test]
fn an_out_of_range_index_or_a_llama_row_does_nothing(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, true);
    view.update(cx, |view, _| {
        view.config.llm_launchers.push(Launcher {
            label: "coder".into(),
            host: "remoteluci".into(),
            group: None,
            kind: Kind::Llama {
                port: 8081,
                start: "s".into(),
                url: None,
            },
        });
    });
    click(&view, cx, 7);
    click(&view, cx, 1);
    assert_eq!(flash(&view, cx), None);
    assert_eq!(job(&view, cx), None);
    assert_eq!(view.read_with(cx, |view, _| view.selected_endpoint), 0);
}

#[gpui::test]
fn a_pending_job_blocks_a_second_click(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, true);
    click(&view, cx, 0);
    let first = job(&view, cx);
    assert!(first.is_some());
    view.update(cx, |view, _| view.flash = None);
    click(&view, cx, 0);
    assert_eq!(job(&view, cx), first);
    assert_eq!(flash(&view, cx), None);
}

#[gpui::test]
fn a_refused_switch_says_so_and_starts_nothing(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, true);
    view.update(cx, |view, _| {
        view.menu.page = Some(crate::menu::Page::About);
    });
    click(&view, cx, 0);
    assert_eq!(
        flash(&view, cx).as_deref(),
        Some("Cannot switch to remoteluci")
    );
    assert_eq!(job(&view, cx), None);
    assert_eq!(view.read_with(cx, |view, _| view.selected_endpoint), 0);
}

#[gpui::test]
fn a_successful_switch_sets_the_job_with_the_exact_line(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, true);
    click(&view, cx, 0);
    assert_eq!(
        job(&view, cx),
        Some(("ssh:remoteluci".to_owned(), RUN.to_owned()))
    );
    assert_eq!(view.read_with(cx, |view, _| view.selected_endpoint), 1);
}

#[gpui::test]
fn an_expired_deadline_fails_with_the_timeout_flash_and_clears_the_job(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, true);
    click(&view, cx, 0);
    view.update(cx, |view, cx| {
        if let Some(job) = &mut view.llm_launch.terminal {
            job.deadline = Instant::now() - Duration::from_secs(1);
        }
        view.poll_llm_terminal(cx);
    });
    assert_eq!(
        flash(&view, cx).as_deref(),
        Some("nemotron-3-nano:4b did not open: the host did not answer in time")
    );
    assert_eq!(job(&view, cx), None);
}

#[gpui::test]
fn polling_without_a_job_does_nothing(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, true);
    view.update(cx, |view, cx| view.poll_llm_terminal(cx));
    assert_eq!(flash(&view, cx), None);
    assert_eq!(job(&view, cx), None);
}

#[gpui::test]
fn a_pending_job_holds_the_script_slot(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, true);
    assert!(!view.read_with(cx, |view, _| view.script_slot_busy()));
    view.update(cx, |view, _| {
        view.llm_launch.terminal = Some(TerminalJob {
            endpoint: "ssh:remoteluci".into(),
            label: LABEL.into(),
            line: RUN.into(),
            deadline: Instant::now() + Duration::from_secs(15),
            boot: None,
            sent: None,
        });
    });
    assert!(view.read_with(cx, |view, _| view.script_slot_busy()));
}

#[gpui::test]
fn the_click_path_still_counts_terminal_runs(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, false);
    let before = TERMINAL_RUNS.with(|runs| runs.get());
    click(&view, cx, 0);
    assert_eq!(TERMINAL_RUNS.with(|runs| runs.get()), before + 1);
}

// ---- the poll, with a faked snapshot ----

fn job_on(endpoint: &str, boot: Option<&str>) -> TerminalJob {
    TerminalJob {
        endpoint: endpoint.into(),
        label: LABEL.into(),
        line: RUN.into(),
        deadline: Instant::now() + Duration::from_secs(15),
        boot: boot.map(str::to_owned),
        sent: None,
    }
}

/// Polls a job on the shown (first) endpoint over a connected snapshot of
/// `workspaces` workspaces.
fn poll_shown(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    workspaces: usize,
    boot: Option<&str>,
) {
    view.update(cx, |view, cx| {
        let mut shown = crate::sidebar::layout_tests::snapshot(workspaces);
        shown.focused_workspace_id = None;
        view.live.snapshot = Some(std::sync::Arc::new(shown));
        view.live.status = crate::state::ConnectionStatus::Connected;
        let id = view.endpoints[view.selected_endpoint].id.clone();
        view.llm_launch.terminal = Some(job_on(&id, boot));
        view.poll_llm_terminal(cx);
    });
}

#[gpui::test]
fn polling_waits_while_another_endpoint_is_shown(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, true);
    view.update(cx, |view, cx| {
        view.llm_launch.terminal = Some(job_on("ssh:remoteluci", None));
        view.poll_llm_terminal(cx);
    });
    assert_eq!(
        job(&view, cx).map(|(endpoint, _)| endpoint).as_deref(),
        Some("ssh:remoteluci")
    );
    assert_eq!(flash(&view, cx), None);
}

#[gpui::test]
fn polling_waits_without_a_snapshot(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, true);
    click(&view, cx, 0);
    view.update(cx, |view, cx| {
        view.live.snapshot = None;
        view.poll_llm_terminal(cx);
    });
    assert!(job(&view, cx).is_some());
    assert_eq!(flash(&view, cx), None);
}

#[gpui::test]
fn a_host_without_a_workspace_fails_the_job(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, false);
    poll_shown(&view, cx, 0, None);
    assert_eq!(
        flash(&view, cx).as_deref(),
        Some("nemotron-3-nano:4b did not open: no workspace on that host")
    );
    assert_eq!(job(&view, cx), None);
}

#[gpui::test]
fn a_boot_that_changed_fails_the_job_as_stale(cx: &mut TestAppContext) {
    let (view, cx) = window(cx, false);
    poll_shown(&view, cx, 2, Some("an-older-boot"));
    let text = flash(&view, cx).unwrap_or_default();
    assert!(
        text.contains(&crate::Error::StaleConnection.to_string()),
        "{text}"
    );
    assert_eq!(job(&view, cx), None);
}
