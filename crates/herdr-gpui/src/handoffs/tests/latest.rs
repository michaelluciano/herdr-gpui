//! The `LATEST.md` and `history/` layout, and the summary line.

use super::*;
use crate::handoffs::{
    Kind,
    classify::{Shelf, classify},
    line::{detail, title},
    list::list_all,
};

fn with_body(id: &str, status: &str, body: &str) -> String {
    format!("---\nid: {id}\ncreated: 2026-11-04T10:30:00\nstatus: {status}\n---\n{body}")
}

fn summary_of(body: &str) -> String {
    let root = Root::new("summary");
    root.file("p/LATEST.md", with_body("x", "open", body));
    list_all(root.path()).remove(0).summary
}

#[test]
fn latest_and_history_are_listed_with_their_kind() {
    let root = Root::new("kinds");
    root.file("p/LATEST.md", with_body("new", "open", "Now.\n"));
    root.file(
        "p/history/20261103T101010-abcd1234.md",
        with_body("old", "superseded", "Then.\n"),
    );
    root.session("p", "legacy.md", with_body("leg", "open", "Before.\n"));
    let mut kinds: Vec<(String, Kind)> = list_all(root.path())
        .into_iter()
        .map(|h| (h.id, h.kind))
        .collect();
    kinds.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        kinds,
        [
            ("leg".to_owned(), Kind::Legacy),
            ("new".to_owned(), Kind::Latest),
            ("old".to_owned(), Kind::History),
        ]
    );
}

#[test]
fn the_summary_skips_headings_and_blank_lines_and_strips_a_bullet() {
    assert_eq!(
        summary_of("\n## Remaining work\n\n- Finish the sidebar\n- second\n"),
        "Finish the sidebar"
    );
    assert_eq!(summary_of("# T\n* starred item\n"), "starred item");
    assert_eq!(summary_of("Plain line\n"), "Plain line");
}

#[test]
fn the_summary_is_empty_without_a_fitting_line_and_is_cut() {
    assert_eq!(summary_of(""), "");
    assert_eq!(summary_of("# only heading\n## another\n"), "");
    assert_eq!(summary_of(&"x".repeat(500)).chars().count(), 120);
}

#[test]
fn history_is_shelved_superseded_unless_picked_up() {
    let root = Root::new("shelve");
    root.file("p/LATEST.md", with_body("new", "open", "x\n"));
    root.file(
        "p/history/20261101T000000-aaaaaaaa.md",
        with_body("sup", "superseded", "x\n"),
    );
    root.file(
        "p/history/20261102T000000-bbbbbbbb.md",
        with_body("stale", "open", "x\n"),
    );
    root.file(
        "p/history/20261103T000000-cccccccc.md",
        with_body("got", "claimed", "x\n"),
    );
    let board = classify(list_all(root.path()));
    let active: Vec<&str> = board.active.iter().map(|h| h.id.as_str()).collect();
    assert_eq!(active, ["new"]);
    let mut shelved: Vec<(&str, Shelf)> = board
        .history
        .iter()
        .map(|s| (s.handoff.id.as_str(), s.shelf))
        .collect();
    shelved.sort_by_key(|s| s.0);
    assert_eq!(
        shelved,
        [
            ("got", Shelf::PickedUp),
            ("stale", Shelf::Superseded),
            ("sup", Shelf::Superseded)
        ]
    );
}

#[test]
fn a_superseded_status_outside_history_stays_hidden() {
    let root = Root::new("hidden");
    root.file("p/LATEST.md", with_body("new", "superseded", "x\n"));
    let board = classify(list_all(root.path()));
    assert!(board.active.is_empty() && board.history.is_empty());
}

#[test]
fn a_legacy_handoff_is_shadowed_by_its_projects_latest() {
    let root = Root::new("shadow");
    root.file("p/LATEST.md", with_body("new", "open", "x\n"));
    root.session("p", "leg.md", with_body("leg", "open", "x\n"));
    root.session("q", "solo.md", with_body("solo", "open", "x\n"));
    let board = classify(list_all(root.path()));
    let mut active: Vec<&str> = board.active.iter().map(|h| h.id.as_str()).collect();
    active.sort_unstable();
    assert_eq!(active, ["new", "solo"]);
    assert_eq!(board.history.len(), 1);
    assert_eq!(board.history[0].handoff.id, "leg");
    assert_eq!(board.history[0].shelf, Shelf::Superseded);
}

#[cfg(unix)]
#[test]
fn a_symlinked_latest_is_skipped() {
    let root = Root::new("latest-link");
    let real = root.file("elsewhere/real.md", with_body("real", "open", "x\n"));
    std::fs::create_dir_all(root.path().join("p")).unwrap();
    std::os::unix::fs::symlink(&real, root.path().join("p/LATEST.md")).unwrap();
    assert!(list_all(root.path()).is_empty());
}

#[test]
fn a_latest_row_shows_the_project_and_stamp_with_summary() {
    let root = Root::new("row");
    root.file("proj/LATEST.md", with_body("S-opaque", "open", "- Do it\n"));
    let latest = list_all(root.path()).remove(0);
    assert_eq!(title(&latest), "proj");
    assert_eq!(detail(&latest, None), "11-04 10:30 \u{b7} Do it");
    let bare = Handoff {
        summary: String::new(),
        ..latest
    };
    assert_eq!(detail(&bare, None), "11-04 10:30");
    let legacy = fixture("S-1", "proj", "2026-11-04T10:30:00", "open");
    assert_eq!(title(&legacy), "S-1");
    assert_eq!(detail(&legacy, None), "proj \u{b7} 11-04 10:30");
}
