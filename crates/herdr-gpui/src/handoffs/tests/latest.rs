//! `LATEST.md` and `history/*.md` beside the legacy `sessions/*.md`.
use super::*;
use crate::handoffs::classify::{Board, Shelf, Shelved, classify};

fn header(id: &str, created: &str, status: &str) -> String {
    format!("---\nid: {id}\ncreated: {created}\nstatus: {status}\n---\nbody\n")
}

fn ids(list: &[Handoff]) -> Vec<&str> {
    let mut ids: Vec<&str> = list.iter().map(|h| h.id.as_str()).collect();
    ids.sort_unstable();
    ids
}

fn split(board: &Board) -> (Vec<&str>, Vec<(&str, Shelf)>) {
    (
        board.active.iter().map(|h| h.id.as_str()).collect(),
        board
            .history
            .iter()
            .map(|Shelved { shelf, handoff }| (handoff.id.as_str(), *shelf))
            .collect(),
    )
}

#[test]
fn latest_md_is_listed() {
    let root = Root::new("latest");
    let path = root.file(
        "p/LATEST.md",
        header("latest", "2026-11-05T10:00:00", "open"),
    );
    let list = list_all(root.path());
    assert_eq!(ids(&list), ["latest"]);
    assert_eq!(list[0].path, path);
    assert_eq!(list[0].project, "p");
}

#[test]
fn history_files_are_listed() {
    let root = Root::new("history");
    root.file(
        "p/history/a-1.md",
        header("h1", "2026-11-04T10:00:00", "open"),
    );
    root.file(
        "p/history/b-2.md",
        header("h2", "2026-11-03T10:00:00", "open"),
    );
    root.file(
        "p/history/notes.txt",
        header("txt", "2026-11-03T10:00:00", "open"),
    );
    assert_eq!(ids(&list_all(root.path())), ["h1", "h2"]);
}

#[cfg(unix)]
#[test]
fn a_symlinked_latest_md_is_skipped() {
    let root = Root::new("latest-link");
    let real = root.file("elsewhere/real.md", header("real", "2026-11-05", "open"));
    root.file("p/history/h.md", header("h", "2026-11-04", "open"));
    std::os::unix::fs::symlink(&real, root.path().join("p/LATEST.md")).unwrap();
    assert_eq!(ids(&list_all(root.path())), ["h"]);
}

#[cfg(unix)]
#[test]
fn a_symlinked_history_dir_is_skipped() {
    let root = Root::new("history-link");
    root.file("elsewhere/x.md", header("x", "2026-11-04", "open"));
    root.file("p/LATEST.md", header("latest", "2026-11-05", "open"));
    std::os::unix::fs::symlink(root.path().join("elsewhere"), root.path().join("p/history"))
        .unwrap();
    assert_eq!(ids(&list_all(root.path())), ["latest"]);
}

#[test]
fn latest_is_active_and_history_and_legacy_are_not() {
    let root = Root::new("e2e");
    root.file(
        "p/LATEST.md",
        header("latest", "2026-11-05T12:00:00", "open"),
    );
    root.file(
        "p/history/a.md",
        header("h-open", "2026-11-05T11:00:00", "open"),
    );
    root.file(
        "p/history/b.md",
        header("h-old", "2026-11-04T11:00:00", "superseded"),
    );
    root.session(
        "p",
        "legacy.md",
        header("legacy", "2026-11-01T09:00:00", "open"),
    );
    let board = classify(list_all(root.path()));
    assert_eq!(
        split(&board),
        (
            vec!["latest"],
            vec![
                ("h-open", Shelf::Superseded),
                ("h-old", Shelf::Superseded),
                ("legacy", Shelf::Superseded),
            ]
        )
    );
}

#[test]
fn a_superseded_status_lands_in_the_superseded_shelf() {
    let board = classify(vec![
        fixture("new", "p", "2026-11-05T12:00:00", "open"),
        fixture("old", "p", "2026-11-04T12:00:00", "superseded"),
    ]);
    assert_eq!(
        split(&board),
        (vec!["new"], vec![("old", Shelf::Superseded)])
    );
}

#[test]
fn a_superseded_status_never_counts_as_the_projects_newest() {
    let board = classify(vec![
        fixture("old", "p", "2026-11-05T12:00:00", "superseded"),
        fixture("open", "p", "2026-11-04T12:00:00", "open"),
    ]);
    assert_eq!(
        split(&board),
        (vec!["open"], vec![("old", Shelf::Superseded)])
    );
}
