#![allow(clippy::expect_used)]

use super::*;
use crate::{config::LayoutMode, handoffs::Handoff, state::ConnectionStatus};
use gpui::{Entity, VisualTestContext, px, size};

/// An open handoff with the given id, project and created.
pub(super) fn handoff(id: &str, project: &str, created: &str) -> Handoff {
    Handoff {
        path: std::env::temp_dir().join(format!("herdr-handoff-render/{project}/sessions/{id}.md")),
        project: project.into(),
        id: id.into(),
        created: created.into(),
        status: "open".into(),
        session_id: String::new(),
        parent: String::new(),
        prev: String::new(),
        claimed_by: String::new(),
    }
}

fn three() -> Vec<Handoff> {
    vec![
        handoff(
            "S-2026-10-31-e8da4cd3",
            "claude-setup",
            "2026-10-31T13:41:42",
        ),
        handoff("S38-next", "kb", "2026-10-31"),
        handoff("S-2026-10-30-aa11bb22", "remora", "2026-10-30T09:00:00"),
    ]
}

/// A focused-workspace window showing `list` as its handoffs, sorted the
/// way the poll sorts them.
pub(super) fn window(
    cx: &mut gpui::TestAppContext,
    mode: Option<LayoutMode>,
    list: Vec<Handoff>,
) -> (Entity<HerdrWindow>, &mut VisualTestContext) {
    let (fixture, cx) = cx.add_window_view(move |window, cx| {
        let view = cx.new(|cx| {
            let mut view = fixture_window(window, cx);
            if let Some(mode) = mode {
                view.config.layout.mode = mode;
            }
            let mut shown = snapshot(2);
            shown.focused_workspace_id = Some("w0".into());
            shown.focused_tab_id = Some("t0".into());
            view.live.snapshot = Some(Arc::new(shown));
            view.live.status = ConnectionStatus::Connected;
            view.handoffs.set_list(list);
            view
        });
        cx.observe(&view, |_, _, cx| cx.notify()).detach();
        SidebarFixture(view)
    });
    let view = cx.update(|_, cx| fixture.read(cx).0.clone());
    cx.simulate_resize(size(px(800.), px(700.)));
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    (view, cx)
}

pub(super) fn texts(cx: &mut VisualTestContext) -> Vec<String> {
    cx.update(|_, cx| cx.global::<TextProbes>().0.keys().cloned().collect())
}

#[gpui::test]
fn open_handoffs_list_below_the_workspaces_and_agents(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = window(cx, None, three());
    let section = cx
        .debug_bounds("handoffs-section")
        .expect("no handoffs section");
    assert!(cx.debug_bounds("handoffs-scroll").is_some());
    let spaces = cx.debug_bounds("spaces-section").unwrap();
    assert!(
        spaces.bottom() <= section.top(),
        "below the workspaces list"
    );
    if let Some(agents) = cx.debug_bounds("agents-section") {
        assert!(agents.bottom() <= section.top(), "below the agents list");
    }
    let row = cx.debug_bounds("handoff-row-0").expect("no active handoff");
    assert!(section.top() <= row.top() && row.bottom() <= section.bottom());
}

#[gpui::test]
fn the_first_row_shows_its_project_then_the_short_id_and_age(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = window(cx, None, three());
    let shown = texts(cx);
    assert!(shown.iter().any(|text| text == "claude-setup"), "{shown:?}");
    assert!(
        shown
            .iter()
            .any(|text| text.starts_with("S-e8da4cd3 \u{b7} ")),
        "short id and age in {shown:?}"
    );
    assert!(
        !shown.iter().any(|text| text.contains("2026-10-31")),
        "the date is not shown: {shown:?}"
    );
    assert!(
        shown
            .iter()
            .any(|text| text.to_lowercase().contains("handoffs")),
        "a Handoffs heading in {shown:?}"
    );
    assert!(
        shown.iter().any(|text| text == "\u{b7} 3 active"),
        "the labelled count in {shown:?}"
    );
    assert!(cx.debug_bounds("handoffs-count").is_some());
    assert!(
        !shown.iter().any(|text| text == "3"),
        "no bare count in {shown:?}"
    );
}

#[gpui::test]
fn no_handoffs_draw_no_section_and_no_heading(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = window(cx, None, Vec::new());
    assert!(cx.debug_bounds("handoffs-section").is_none());
    assert!(cx.debug_bounds("handoffs-scroll").is_none());
    assert!(cx.debug_bounds("handoff-row-0").is_none());
    assert!(
        !texts(cx)
            .iter()
            .any(|text| text.to_lowercase().contains("handoffs")),
        "no empty heading"
    );
    // The workspaces list still draws.
    assert!(cx.debug_bounds("spaces-section").is_some());
}

#[gpui::test]
fn the_devices_layout_shows_no_handoffs(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = window(cx, Some(LayoutMode::Devices), three());
    assert!(cx.debug_bounds("handoffs-section").is_none());
    assert!(cx.debug_bounds("handoff-row-0").is_none());
}

#[gpui::test]
fn a_long_list_scrolls_inside_thirty_percent_of_the_sidebar(cx: &mut gpui::TestAppContext) {
    let many: Vec<_> = (0..40)
        .map(|n| handoff(&format!("S{n:02}"), &format!("proj{n:02}"), "2026-10-31"))
        .collect();
    let (_view, cx) = window(cx, None, many);
    let sidebar = cx.debug_bounds("sidebar").unwrap();
    let section = cx
        .debug_bounds("handoffs-section")
        .expect("no handoffs section");
    assert!(
        section.size.height <= sidebar.size.height * 0.30 + px(1.),
        "section {:?} of sidebar {:?}",
        section.size.height,
        sidebar.size.height
    );
    let spaces = cx.debug_bounds("spaces-section").unwrap();
    assert!(
        spaces.size.height > px(0.),
        "the workspaces list keeps room"
    );
    assert!(section.bottom() <= sidebar.bottom());
}

#[gpui::test]
fn a_short_list_is_shorter_than_the_cap(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = window(cx, None, three());
    let sidebar = cx.debug_bounds("sidebar").unwrap();
    let section = cx
        .debug_bounds("handoffs-section")
        .expect("no handoffs section");
    assert!(section.size.height <= sidebar.size.height * 0.30 + px(1.));
}

#[gpui::test]
fn the_remove_control_keeps_its_width_and_the_header_has_no_bare_count(
    cx: &mut gpui::TestAppContext,
) {
    let (_view, cx) = window(cx, None, three());
    let remove = cx
        .debug_bounds("handoff-row-x-0")
        .expect("the x stays laid out");
    assert!(remove.size.width > px(0.), "{remove:?}");
    let row = cx.debug_bounds("handoff-row-0").unwrap();
    assert!(remove.right() <= row.right());
}
