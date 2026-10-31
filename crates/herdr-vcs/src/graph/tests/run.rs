use super::*;
use std::{
    path::Path,
    time::{Duration, Instant},
};

fn git(dir: &Path, args: &[&str]) {
    let output = std::process::Command::new("git")
        .args([
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "-c",
            "commit.gpgsign=false",
            "-C",
        ])
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}

#[test]
fn a_real_repository_with_a_branch_a_merge_and_a_tag_lays_out_in_lanes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    git(root, &["init", "-q", "-b", "main"]);
    std::fs::write(root.join("one"), "1").unwrap();
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "base"]);
    git(root, &["tag", "v1"]);
    git(root, &["checkout", "-q", "-b", "feature"]);
    std::fs::write(root.join("side"), "s").unwrap();
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "side work"]);
    git(root, &["checkout", "-q", "main"]);
    std::fs::write(root.join("two"), "2").unwrap();
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "main work"]);
    git(
        root,
        &["merge", "-q", "--no-ff", "-m", "merge feature", "feature"],
    );

    let commits = log(root.to_str().unwrap(), 100, deadline(), &|| false).unwrap();
    let merge = commits.iter().position(|c| c.parents.len() == 2).unwrap();
    assert_eq!(commits[merge].subject, "merge feature");
    assert!(commits[merge].refs.contains(&Ref::Head));
    assert!(
        commits[merge]
            .refs
            .contains(&Ref::LocalBranch("main".into()))
    );
    assert!(
        commits
            .iter()
            .any(|c| c.refs.contains(&Ref::Tag("v1".into())))
    );
    assert!(commits.iter().any(|c| c.parents.is_empty()));

    let layout = layout(&commits);
    assert!(layout.lane_count >= 2, "{layout:?}");
    assert_eq!(layout.rows[merge].edges.len(), 2);
    assert!(layout.rows.iter().all(|row| row.lane < layout.lane_count));
}

#[test]
fn the_window_is_limited_and_validated() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    git(root, &["init", "-q", "-b", "main"]);
    for n in 0..3 {
        git(
            root,
            &["commit", "-q", "--allow-empty", "-m", &format!("c{n}")],
        );
    }
    let path = root.to_str().unwrap();
    let two = log(path, 2, deadline(), &|| false).unwrap();
    assert_eq!(two.len(), 2);
    // The window ends mid-history, so the oldest returned commit dangles.
    let laid = layout(&two);
    assert_eq!(laid.rows[1].edges[0].kind, EdgeKind::Dangling);

    for bad in [0, MAX_LIMIT + 1] {
        assert!(matches!(
            log(path, bad, deadline(), &|| false),
            Err(crate::Error::Graph(Error::Limit { .. }))
        ));
    }
}

#[test]
fn a_directory_that_is_not_a_repository_fails_typed() {
    let temp = tempfile::tempdir().unwrap();
    let result = log(temp.path().to_str().unwrap(), 10, deadline(), &|| false);
    assert!(matches!(
        result,
        Err(crate::Error::Git(crate::git::Error::Failed { .. }))
    ));
}

#[test]
fn a_cancelled_request_does_not_run_git() {
    let temp = tempfile::tempdir().unwrap();
    let result = log(temp.path().to_str().unwrap(), 10, deadline(), &|| true);
    assert!(result.is_err());
}
