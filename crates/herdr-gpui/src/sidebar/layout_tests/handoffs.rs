#![allow(clippy::expect_used)]

use super::*;
use crate::{
    browser::{Location, Store},
    config::LayoutMode,
    handoffs::Handoff,
    state::ConnectionStatus,
};
use gpui::{Entity, Modifiers, VisualTestContext, px, size};

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
            view.handoffs.board = crate::handoffs::classify(list);
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
    let rows: Vec<_> = (0..3)
        .map(|n| {
            let name: &'static str = Box::leak(format!("handoff-row-{n}").into_boxed_str());
            cx.debug_bounds(name).unwrap_or_else(|| panic!("no {name}"))
        })
        .collect();
    for pair in rows.windows(2) {
        assert!(pair[0].bottom() <= pair[1].top(), "rows stack in order");
    }
    for row in &rows {
        assert!(section.top() <= row.top() && row.bottom() <= section.bottom());
    }
    assert!(cx.debug_bounds("handoff-row-3").is_none());
}

#[gpui::test]
fn each_row_shows_its_id_and_project_with_the_written_time(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = window(cx, None, three());
    let shown = texts(cx);
    for expected in [
        "S-2026-10-31-e8da4cd3",
        "claude-setup \u{b7} 10-31 13:41",
        "S38-next",
        "kb \u{b7} 10-31",
        "S-2026-10-30-aa11bb22",
        "remora \u{b7} 10-30 09:00",
    ] {
        assert!(
            shown.iter().any(|text| text == expected),
            "{expected:?} in {shown:?}"
        );
    }
    assert!(
        shown
            .iter()
            .any(|text| text.to_lowercase().contains("handoffs")),
        "a Handoffs heading in {shown:?}"
    );
    assert!(shown.iter().any(|text| text.contains('3')), "the count");
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
        .map(|n| handoff(&format!("S{n:02}"), "proj", "2026-10-31"))
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
fn clicking_a_row_opens_that_file_in_a_code_tab(cx: &mut gpui::TestAppContext) {
    let list = three();
    let wanted = list[1].path.to_str().unwrap().to_owned();
    let (_view, cx) = window(cx, None, list);
    let row = cx.debug_bounds("handoff-row-1").expect("no second row");
    cx.simulate_click(row.center(), Modifiers::default());
    cx.run_until_parked();
    let opened: Vec<String> = cx.update(|_, cx| {
        cx.try_global::<Store>().map_or_else(Vec::new, |store| {
            store
                .tabs()
                .filter_map(|tab| match &tab.location {
                    Some(Location::Code { file }) => Some(file.path.clone()),
                    _ => None,
                })
                .collect()
        })
    });
    assert_eq!(opened, [wanted]);
}
