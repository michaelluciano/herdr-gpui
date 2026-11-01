use super::tab::{draw, open, window};
use crate::{
    HerdrWindow,
    browser::{Store, TabId},
    graph_view::load::GRAPH_REFRESH,
};
use gpui::{Entity, Modifiers, VisualTestContext};
use std::{path::Path, process::Command, time::Duration, time::Instant};

fn git(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .args(["-c", "user.name=T", "-c", "user.email=t@example.com"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success(), "git {args:?}: {output:?}");
}

fn commit_file(dir: &Path, name: &str) {
    std::fs::write(dir.join(name), name).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-m", &format!("add {name}")]);
}

/// Three commits: add a.txt, add b.txt, rename a.txt to c.txt.
fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "main"]);
    commit_file(dir.path(), "a.txt");
    commit_file(dir.path(), "b.txt");
    git(dir.path(), &["mv", "a.txt", "c.txt"]);
    git(dir.path(), &["commit", "-q", "-m", "rename a to c"]);
    dir
}

fn graph_id(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> TabId {
    view.read_with(cx, |view, _| *view.graphs.keys().next().unwrap())
}

fn poll(view: &Entity<HerdrWindow>, now: Instant, cx: &mut VisualTestContext) {
    cx.update(|_, cx| view.update(cx, |view, cx| view.poll_graphs_at(now, cx)));
}

/// Let background work land, redrawing until `done` holds (bounded, no sleeps).
fn settle(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    done: impl Fn(&HerdrWindow) -> bool,
) {
    cx.executor().allow_parking();
    for _ in 0..200 {
        cx.run_until_parked();
        if view.read_with(cx, |view, _| done(view)) {
            break;
        }
    }
    draw(cx);
}

fn open_loaded(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> TabId {
    open(view, cx);
    poll(view, Instant::now(), cx);
    settle(view, cx, |view| {
        view.graphs
            .values()
            .next()
            .is_some_and(|s| s.commits.len() == 3)
    });
    graph_id(view, cx)
}

fn first_file(view: &Entity<HerdrWindow>, id: TabId, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |view, _| {
        let files = view.graphs[&id].files.as_ref()?;
        files.changes.first().map(|change| change.path.clone())
    })
}

fn commit_count(view: &Entity<HerdrWindow>, id: TabId, cx: &mut VisualTestContext) -> usize {
    view.read_with(cx, |view, _| view.graphs[&id].commits.len())
}

#[gpui::test]
fn opening_a_graph_on_a_repo_loads_and_draws_its_commits(cx: &mut gpui::TestAppContext) {
    let dir = repo();
    let (view, cx) = window(cx, Some(dir.path()));
    let id = open_loaded(&view, cx);
    assert_eq!(commit_count(&view, id, cx), 3);
    for row in ["graph-row-0", "graph-row-1", "graph-row-2"] {
        assert!(cx.debug_bounds(row).is_some(), "no {row}");
    }
}

#[gpui::test]
fn the_detail_pane_lists_the_head_commits_files(cx: &mut gpui::TestAppContext) {
    let dir = repo();
    let (view, cx) = window(cx, Some(dir.path()));
    let id = open_loaded(&view, cx);
    settle(&view, cx, |view| {
        view.graphs.values().all(|s| s.files.is_some())
    });
    assert!(cx.debug_bounds("graph-file-0").is_some(), "no changed file");
    assert!(cx.debug_bounds("graph-files-loading").is_none());
    assert!(first_file(&view, id, cx).is_some());
}

#[gpui::test]
fn clicking_a_row_loads_that_commits_files(cx: &mut gpui::TestAppContext) {
    let dir = repo();
    let (view, cx) = window(cx, Some(dir.path()));
    let id = open_loaded(&view, cx);
    settle(&view, cx, |view| {
        view.graphs.values().all(|s| s.files.is_some())
    });
    let before = first_file(&view, id, cx);
    assert!(before.is_some(), "the head commit's files never loaded");

    let row = cx.debug_bounds("graph-row-1").unwrap();
    cx.simulate_click(row.center(), Modifiers::default());
    settle(&view, cx, |view| {
        view.graphs.values().all(|s| s.files.is_some())
            && view
                .graphs
                .values()
                .all(|s| s.files.as_ref().map(|f| f.commit.as_str()) == s.selected.as_deref())
    });
    let after = first_file(&view, id, cx);
    assert!(after.is_some());
    assert_ne!(before, after);
    assert!(cx.debug_bounds("graph-file-0").is_some());
}

#[gpui::test]
fn a_new_commit_appears_only_after_the_refresh_interval(cx: &mut gpui::TestAppContext) {
    let dir = repo();
    let (view, cx) = window(cx, Some(dir.path()));
    let id = open_loaded(&view, cx);
    assert_eq!(commit_count(&view, id, cx), 3);
    let loaded = view.read_with(cx, |view, _| view.graphs[&id].last_load);
    assert!(loaded.is_some(), "the first load never marked its time");
    let loaded = loaded.unwrap();

    commit_file(dir.path(), "d.txt");
    poll(&view, loaded + Duration::from_secs(1), cx);
    settle(&view, cx, |_| false);
    assert_eq!(
        commit_count(&view, id, cx),
        3,
        "reloaded before the interval"
    );

    poll(&view, loaded + GRAPH_REFRESH, cx);
    settle(&view, cx, |view| {
        view.graphs.values().all(|s| s.commits.len() == 4)
    });
    assert_eq!(commit_count(&view, id, cx), 4);
}

#[gpui::test]
fn a_folder_that_is_not_a_repo_shows_the_empty_state(cx: &mut gpui::TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (view, cx) = window(cx, Some(dir.path()));
    open(&view, cx);
    let now = Instant::now();
    poll(&view, now, cx);
    settle(&view, cx, |view| {
        view.graphs.values().all(|s| s.last_load.is_some())
    });
    let id = graph_id(&view, cx);
    assert!(
        view.read_with(cx, |view, _| view.graphs[&id].last_load.is_some()),
        "no load was attempted"
    );
    assert_eq!(commit_count(&view, id, cx), 0);
    assert!(cx.debug_bounds("graph-empty").is_some());
    poll(&view, now + GRAPH_REFRESH * 2, cx);
    settle(&view, cx, |_| false);
    assert_eq!(commit_count(&view, id, cx), 0);
}

#[gpui::test]
fn closing_the_tab_cancels_a_load_in_flight(cx: &mut gpui::TestAppContext) {
    let dir = repo();
    let (view, cx) = window(cx, Some(dir.path()));
    open(&view, cx);
    poll(&view, Instant::now(), cx);
    let id = graph_id(&view, cx);
    let (request, flag) = view.read_with(cx, |view, _| {
        let state = &view.graphs[&id];
        (state.request, state.cancel.clone())
    });
    assert!(request >= 1, "no load was started");
    assert!(!flag.load(std::sync::atomic::Ordering::SeqCst));
    cx.update(|_, cx| Store::update(cx, |store| store.close(id)));
    cx.update(|_, cx| view.update(cx, |view, cx| view.poll_graphs(cx)));
    assert!(flag.load(std::sync::atomic::Ordering::SeqCst));
}
