//! Which open handoff belongs to the focused workspace's branch or checkout.
use super::*;

fn on(id: &str, created: &str, branch: &str, cwd: &str) -> Handoff {
    Handoff {
        branch: branch.into(),
        cwd: cwd.into(),
        ..fixture(id, "p", created, "open")
    }
}

fn current_id<'a>(active: &'a [Handoff], branch: Option<&str>, cwd: &str) -> Option<&'a str> {
    current(active, branch, cwd).map(|handoff| handoff.id.as_str())
}

#[test]
fn the_newest_handoff_on_the_branch_is_current() {
    // Newest first, as the loader sorts them.
    let active = [
        on("other", "2026-11-01T10:00", "fix/x", ""),
        on("newer", "2026-11-01T09:00", "main", ""),
        on("older", "2026-10-31T09:00", "main", ""),
    ];
    assert_eq!(current_id(&active, Some("main"), "/repo"), Some("newer"));
    assert_eq!(current_id(&active, Some("fix/x"), "/repo"), Some("other"));
    assert_eq!(current_id(&active, Some("develop"), "/repo"), None);
    assert_eq!(current_id(&active, None, "/repo"), None);
}

#[test]
fn a_handoff_with_a_cwd_must_sit_in_the_workspace_checkout() {
    let active = [
        on("elsewhere", "2026-11-01T10:00", "main", "/other/repo"),
        on("inside", "2026-11-01T09:00", "main", "/repo/crates/x"),
    ];
    assert_eq!(current_id(&active, Some("main"), "/repo"), Some("inside"));
    assert_eq!(current_id(&active, Some("main"), "/repo/"), Some("inside"));
    // A cwd alone ties a handoff to its checkout whatever the branch.
    let cwd_only = [on("wt", "2026-11-01T09:00", "", "/wt/feature")];
    assert_eq!(
        current_id(&cwd_only, Some("feature"), "/wt/feature"),
        Some("wt")
    );
    assert_eq!(current_id(&cwd_only, None, "/wt/feature"), Some("wt"));
    assert_eq!(current_id(&cwd_only, Some("feature"), "/wt/other"), None);
}

#[test]
fn a_handoff_naming_neither_branch_nor_cwd_is_never_current() {
    let active = [on("bare", "2026-11-01T09:00", "", "")];
    assert_eq!(current_id(&active, Some("main"), "/repo"), None);
    assert_eq!(current_id(&active, Some(""), ""), None);
}

#[test]
fn the_loader_reads_branch_and_cwd() {
    let root = tempfile::tempdir().unwrap();
    let sessions = root.path().join("remora").join("sessions");
    std::fs::create_dir_all(&sessions).unwrap();
    std::fs::write(
        sessions.join("S1.md"),
        "---\nid: S1\ncreated: 2026-11-01T09:00\nstatus: open\nbranch: fix/x\ncwd: /home/u/remora\n---\nbody\n",
    )
    .unwrap();
    let list = list_all(root.path());
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].branch, "fix/x");
    assert_eq!(list[0].cwd, "/home/u/remora");
}
