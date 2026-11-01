#![allow(clippy::expect_used)]

use super::*;
use crate::{
    config::{
        LayoutMode,
        llm_launchers::{Kind, Launcher},
    },
    llm_launch::{Health, Observed},
    state::ConnectionStatus,
    window::Flash,
};
use gpui::{Bounds, Entity, Modifiers, Pixels, VisualTestContext, point, px, size};

const LOCAL: &str = "local";
const REMOTE: &str = "ssh:test";

fn llama(label: &str, port: u16) -> Launcher {
    Launcher {
        label: label.into(),
        host: "Local".into(),
        group: Some("mac-llm".into()),
        kind: Kind::Llama {
            port,
            start: "true".into(),
            url: None,
        },
    }
}

/// Three local llama rows in one group and a terminal row on remoteluci.
fn launchers() -> Vec<Launcher> {
    vec![
        llama("coder", 8081),
        llama("reason", 8082),
        llama("gemma", 8083),
        Launcher {
            label: "nemotron".into(),
            host: "remoteluci".into(),
            group: None,
            kind: Kind::Terminal { run: "true".into() },
        },
    ]
}

fn observed(health: [Option<Health>; 4], foreign: Vec<u16>) -> Observed {
    Observed {
        health: health.to_vec(),
        foreign,
    }
}

fn idle() -> Observed {
    observed([None; 4], vec![])
}

fn remote_endpoint(workspaces: usize) -> crate::endpoint::Endpoint {
    let mut remote = crate::endpoint::Endpoint::new(
        REMOTE.into(),
        "remoteluci".into(),
        ConnectTarget::Ssh {
            target: "unused".into(),
            session: "default".into(),
        },
        true,
    );
    remote.live.snapshot = Some(Arc::new(snapshot(workspaces)));
    remote
}

/// Two hosts, each with `workspaces` workspaces, the four launcher rows, and
/// `observed` injected; `setup` adjusts the window before the first draw.
fn open(
    cx: &mut gpui::TestAppContext,
    workspaces: usize,
    observed: Observed,
    setup: impl FnOnce(&mut HerdrWindow) + 'static,
) -> (Entity<HerdrWindow>, &mut VisualTestContext) {
    let (view, cx) = cx.add_window_view(move |window, cx| {
        let mut view = fixture_window(window, cx);
        crate::bind_keys(cx);
        view.live.snapshot = Some(Arc::new(snapshot(workspaces)));
        view.live.status = ConnectionStatus::Connected;
        view.live.supports_surface = true;
        view.endpoints[0].live = view.live.clone();
        view.endpoints.push(remote_endpoint(workspaces));
        view.config.llm_launchers = launchers();
        view.llm_launch.observed = observed;
        setup(&mut view);
        view
    });
    cx.simulate_resize(size(px(800.), px(700.)));
    cx.run_until_parked();
    for _ in 0..2 {
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
    }
    (view, cx)
}

/// Selectors are `'static`, so a test's formatted ones are leaked.
fn leak(selector: String) -> &'static str {
    Box::leak(selector.into_boxed_str())
}

fn row(host: &str, index: usize) -> &'static str {
    leak(format!("llm-row-{host}-{index}"))
}

fn marker(kind: &str, host: &str, index: usize) -> &'static str {
    leak(format!("llm-row-{kind}-{host}-{index}"))
}

fn present(cx: &mut VisualTestContext, host: &str, index: usize) -> bool {
    cx.debug_bounds(row(host, index)).is_some()
}

fn has(cx: &mut VisualTestContext, kind: &str, host: &str, index: usize) -> bool {
    cx.debug_bounds(marker(kind, host, index)).is_some()
}

fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("missing {selector}"))
}

fn flash_text(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |view, _| {
        view.flash.as_ref().map(|(flash, _)| flash.text.to_string())
    })
}

fn plan_runs() -> usize {
    super::super::llm_rows::PLAN_RUNS.with(|runs| runs.get())
}

fn click(index: usize, view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.click_llm_launcher(index, window, cx));
    });
}

#[gpui::test]
fn local_rows_sit_below_their_header_above_the_workspaces_in_config_order(
    cx: &mut gpui::TestAppContext,
) {
    let (_view, cx) = open(cx, 2, idle(), |_| {});
    let header = bounds(cx, "host-local");
    let first_workspace = bounds(cx, "workspace-local-w0");
    let mut above = header.bottom();
    for index in 0..3 {
        let row = bounds(cx, row(LOCAL, index));
        assert!(above <= row.top(), "row {index} below what precedes it");
        above = row.bottom();
    }
    assert!(above <= first_workspace.top(), "rows above the workspaces");
    assert!(!present(cx, LOCAL, 3), "the remote row is not a local row");
}

#[gpui::test]
fn the_remote_row_sits_below_the_remote_header_and_hosts_stay_in_order(
    cx: &mut gpui::TestAppContext,
) {
    let (_view, cx) = open(cx, 2, idle(), |_| {});
    let local_last_row = bounds(cx, row(LOCAL, 2));
    let local_last_workspace = bounds(cx, "workspace-local-w1");
    let header = bounds(cx, "host-ssh:test");
    let remote_row = bounds(cx, row(REMOTE, 3));
    let remote_workspace = bounds(cx, "workspace-ssh:test-w0");
    assert!(local_last_row.bottom() <= local_last_workspace.top());
    assert!(local_last_workspace.bottom() <= header.top());
    assert!(header.bottom() <= remote_row.top());
    assert!(remote_row.bottom() <= remote_workspace.top());
    for index in 0..3 {
        assert!(!present(cx, REMOTE, index), "local rows are not remote");
    }
}

#[gpui::test]
fn a_row_steps_in_from_its_header_by_the_nest_and_child_indents(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = open(cx, 2, idle(), |_| {});
    let look =
        super::super::layout::for_mode(LayoutMode::from(crate::config::Density::Comfortable));
    let step = look.content_x() + look.nest_indent() + look.density.child_indent();
    let header = bounds(cx, "host-local");
    for (host, index) in [(LOCAL, 0), (REMOTE, 3)] {
        let row = bounds(cx, row(host, index));
        let label = bounds(cx, marker("label", host, index));
        assert!(
            label.left() >= row.left() + px(step) - px(0.5),
            "{host}-{index}: {:?} vs {:?}",
            label.left(),
            row.left()
        );
        assert!(label.left() >= header.left() + px(look.density.child_indent()));
    }
}

#[gpui::test]
fn a_row_label_starts_no_further_left_than_the_workspace_names_under_its_host(
    cx: &mut gpui::TestAppContext,
) {
    let (_view, cx) = open(cx, 2, idle(), |_| {});
    for (host, index) in [(LOCAL, 0), (REMOTE, 3)] {
        let label = bounds(cx, marker("label", host, index));
        let name = bounds(cx, "name-herdr");
        assert!(
            label.left() >= name.left(),
            "{host}-{index}: label {:?} vs workspace name {:?}",
            label.left(),
            name.left()
        );
    }
}

#[gpui::test]
fn rows_hide_with_a_collapsed_host(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = open(cx, 2, idle(), |_| {});
    assert!(present(cx, REMOTE, 3), "shown while expanded");
    let (_view, cx) = open(cx, 2, idle(), |view| view.endpoints[1].collapsed = true);
    assert!(!present(cx, REMOTE, 3), "hidden while collapsed");
    assert!(present(cx, LOCAL, 0), "other hosts keep theirs");
}

#[gpui::test]
fn rows_hide_when_one_endpoint_means_no_host_headers(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = open(cx, 2, idle(), |_| {});
    assert!(present(cx, LOCAL, 0), "shown with two hosts");
    let (_view, cx) = open(cx, 2, idle(), |view| view.endpoints.truncate(1));
    assert!(cx.debug_bounds("host-local").is_none(), "not headed");
    assert!(!present(cx, LOCAL, 0));
}

#[gpui::test]
fn rows_hide_in_the_devices_layout(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = open(cx, 2, idle(), |_| {});
    assert!(present(cx, LOCAL, 0), "shown in the default layout");
    let (_view, cx) = open(cx, 2, idle(), |view| {
        view.config.layout.mode = LayoutMode::Devices;
    });
    for host in [LOCAL, REMOTE] {
        for index in 0..4 {
            assert!(!present(cx, host, index), "{host}-{index}");
        }
    }
}

#[gpui::test]
fn rows_hide_while_a_search_replaces_the_list(cx: &mut gpui::TestAppContext) {
    let (view, cx) = open(cx, 2, idle(), |_| {});
    assert!(present(cx, LOCAL, 0), "shown before the search");
    cx.update(|window, cx| {
        let focus = view.read(cx).sidebar_search.input.read(cx).focus.clone();
        window.focus(&focus, cx);
        full_draw(window, cx).clear(cx);
    });
    cx.simulate_input("sidebar-child");
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("spaces-scroll").is_none());
    assert!(cx.debug_bounds("sidebar-search-results").is_some());
    assert!(!present(cx, LOCAL, 0));
}

#[gpui::test]
fn a_running_row_blocks_the_rest_of_its_group_and_only_that_group(cx: &mut gpui::TestAppContext) {
    let health = [Some(Health::Ready), None, None, None];
    let (_view, cx) = open(cx, 2, observed(health, vec![]), |_| {});
    assert!(present(cx, LOCAL, 0), "the running row is listed");
    assert!(has(cx, "suffix", LOCAL, 0));
    assert!(!has(cx, "blocked", LOCAL, 0));
    for index in [1, 2] {
        assert!(has(cx, "blocked", LOCAL, index), "row {index} blocked");
    }
    assert!(!has(cx, "blocked", REMOTE, 3));
    cx.update(|_, cx| {
        assert!(cx.global::<TextProbes>().0.contains_key("running"));
    });
}

#[gpui::test]
fn a_stopped_row_in_a_free_group_is_not_blocked(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = open(cx, 2, idle(), |_| {});
    for index in 0..3 {
        assert!(present(cx, LOCAL, index), "row {index} listed");
        assert!(!has(cx, "blocked", LOCAL, index));
        assert!(!has(cx, "suffix", LOCAL, index));
    }
}

#[gpui::test]
fn a_foreign_model_blocks_every_local_row_and_not_the_remote_one(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = open(cx, 2, observed([None; 4], vec![8090]), |_| {});
    for index in 0..3 {
        assert!(has(cx, "blocked", LOCAL, index), "row {index}");
    }
    assert!(present(cx, REMOTE, 3));
    assert!(!has(cx, "blocked", REMOTE, 3));
}

#[gpui::test]
fn the_remote_row_is_never_blocked_by_running_local_rows(cx: &mut gpui::TestAppContext) {
    let health = [Some(Health::Ready), Some(Health::Ready), None, None];
    let (_view, cx) = open(cx, 2, observed(health, vec![8090]), |_| {});
    assert!(present(cx, REMOTE, 3));
    assert!(!has(cx, "blocked", REMOTE, 3));
}

#[gpui::test]
fn a_loading_row_shows_the_starting_suffix(cx: &mut gpui::TestAppContext) {
    let health = [Some(Health::Loading), None, None, None];
    let (_view, cx) = open(cx, 2, observed(health, vec![]), |_| {});
    assert!(has(cx, "suffix", LOCAL, 0));
    cx.update(|_, cx| {
        assert!(cx.global::<TextProbes>().0.contains_key("starting"));
    });
}

const WORKSPACES: usize = 12;
const PAST: f32 = 5.;

#[gpui::test]
fn the_pinned_header_and_second_host_stay_right_with_rows_present(cx: &mut gpui::TestAppContext) {
    let (view, cx) = open(cx, WORKSPACES, idle(), |_| {});
    let list = bounds(cx, "spaces-scroll");
    let local_rows = bounds(cx, row(LOCAL, 2));
    let local_header = bounds(cx, "host-local");
    let remote_header = bounds(cx, "host-ssh:test");
    // The remote header sits after the local header, its three rows and all
    // twelve workspaces.
    assert!(local_header.bottom() <= local_rows.top());
    let last_workspace = bounds(cx, leak(format!("workspace-local-w{}", WORKSPACES - 1)));
    assert!(local_rows.bottom() <= bounds(cx, "workspace-local-w0").top());
    assert!(last_workspace.bottom() <= remote_header.top());

    let offset = remote_header.top() - list.top() + px(PAST);
    cx.update(|window, cx| {
        let scroll = view.read(cx).sidebar_scroll[0].clone();
        scroll.set_offset(point(px(0.), -offset));
        full_draw(window, cx).clear(cx);
    });
    let top = bounds(cx, "spaces-scroll").top();
    let pinned = bounds(cx, "sticky-host-ssh:test");
    assert!((pinned.top() - top).abs() < px(0.5));
    assert!(cx.debug_bounds("sticky-host-local").is_none());
}

#[gpui::test]
fn clicking_a_blocked_row_flashes_why_and_runs_nothing(cx: &mut gpui::TestAppContext) {
    let health = [Some(Health::Ready), None, None, None];
    let (view, cx) = open(cx, 2, observed(health, vec![]), |_| {});
    let before = plan_runs();
    click(1, &view, cx);
    assert_eq!(
        flash_text(&view, cx).as_deref(),
        Some("coder is running. Stop it first.")
    );
    assert_eq!(plan_runs(), before);
}

#[gpui::test]
fn clicking_a_row_blocked_by_a_foreign_model_names_its_port(cx: &mut gpui::TestAppContext) {
    let (view, cx) = open(cx, 2, observed([None; 4], vec![8090]), |_| {});
    click(0, &view, cx);
    assert_eq!(
        flash_text(&view, cx).as_deref(),
        Some("Another model is running on port 8090. Stop it first.")
    );
}

#[gpui::test]
fn clicking_a_loading_row_flashes_that_it_is_loading(cx: &mut gpui::TestAppContext) {
    let health = [Some(Health::Loading), None, None, None];
    let (view, cx) = open(cx, 2, observed(health, vec![]), |_| {});
    let before = plan_runs();
    click(0, &view, cx);
    assert_eq!(
        flash_text(&view, cx).as_deref(),
        Some("coder is still loading.")
    );
    assert_eq!(plan_runs(), before);
}

#[gpui::test]
fn clicking_the_row_element_reaches_the_same_handler(cx: &mut gpui::TestAppContext) {
    let health = [Some(Health::Ready), None, None, None];
    let (view, cx) = open(cx, 2, observed(health, vec![]), |_| {});
    let target = bounds(cx, row(LOCAL, 1)).center();
    cx.simulate_click(target, Modifiers::none());
    assert_eq!(
        flash_text(&view, cx).as_deref(),
        Some("coder is running. Stop it first.")
    );
}

#[gpui::test]
fn stopped_and_running_rows_route_to_the_plan_runner(cx: &mut gpui::TestAppContext) {
    let health = [Some(Health::Ready), None, None, None];
    let (view, cx) = open(cx, 2, observed(health, vec![]), |_| {});
    let before = plan_runs();
    click(0, &view, cx); // Running -> Open
    click(3, &view, cx); // Stopped -> StartThenOpen
    assert_eq!(plan_runs(), before + 2);
    assert_eq!(flash_text(&view, cx), None);
}

#[gpui::test]
fn an_out_of_range_index_does_nothing(cx: &mut gpui::TestAppContext) {
    let (view, cx) = open(cx, 2, idle(), |_| {});
    let before = plan_runs();
    click(99, &view, cx);
    assert_eq!(flash_text(&view, cx), None);
    assert_eq!(plan_runs(), before);
    let _ = Flash::hint("unused");
}
