//! The second line of an agent row: tab number, separator and status.
use super::*;
use crate::sidebar::{
    agent_line::{tab_tag, widths},
    agents::tab_position,
    layouts::parts::lead_share,
};

#[test]
fn a_tab_reads_as_its_position_among_its_workspace_tabs() {
    let mut snapshot = layout_tests::snapshot(2);
    snapshot.agents[0].tab_id = "t1".into();
    assert_eq!(tab_position(&snapshot.agents[0], &snapshot), Some(2));
    snapshot.agents[0].tab_id = "t0".into();
    assert_eq!(tab_position(&snapshot.agents[0], &snapshot), Some(1));
    // A tab of another workspace does not count, and an unknown one has none.
    snapshot.tabs[0].workspace_id = "w9".into();
    assert_eq!(tab_position(&snapshot.agents[0], &snapshot), None);
    snapshot.agents[0].tab_id = "gone".into();
    assert_eq!(tab_position(&snapshot.agents[0], &snapshot), None);
    assert_eq!(tab_tag(3), "[3]");
}

#[test]
fn the_status_clips_before_the_icon_separator_and_tab() {
    let glyph = 7.;
    // Room for everything: separator, tab and the whole status.
    let all = widths(400., glyph, Some(3), 0., 14);
    assert_eq!((all.sep, all.tab, all.status), (7., 21., 98.));
    // Short room: the status gives way first and the others stay whole.
    let tight = widths(70., glyph, Some(3), 0., 14);
    assert_eq!((tight.sep, tight.tab), (7., 21.));
    assert_eq!(tight.status, 70. - 7. - 21. - 2. * glyph);
    // No room at all: still never negative, never shrinking the tab.
    let none = widths(10., glyph, Some(3), 0., 14);
    assert_eq!((none.sep, none.tab, none.status), (7., 21., 0.));
    // No tab: no separator either.
    let bare = widths(100., glyph, None, 0., 4);
    assert_eq!((bare.sep, bare.tab, bare.status), (0., 0., 28.));
}

#[test]
fn a_name_keeps_twelve_characters_before_a_status_clips() {
    assert_eq!(lead_share(400., 7.), 0.4);
    assert!((lead_share(160., 7.) - 84. / 160.).abs() < 1e-6);
    assert_eq!(lead_share(0., 7.), 0.4);
    assert_eq!(lead_share(50., 7.), 1.);
}

#[test]
fn the_status_mark_never_clips_and_the_text_after_it_does() {
    let glyph = 7.;
    let marked = widths(100., glyph, Some(3), 12., 14);
    assert_eq!((marked.sep, marked.tab, marked.icon), (7., 21., 12.));
    assert_eq!(marked.status, 100. - 7. - 21. - 12. - 3. * glyph);
    let squeezed = widths(30., glyph, Some(3), 12., 14);
    assert_eq!((squeezed.icon, squeezed.status), (12., 0.));
    // A mark with no text after it takes no trailing gap.
    assert_eq!(widths(100., glyph, None, 12., 0).status, 0.);
}
