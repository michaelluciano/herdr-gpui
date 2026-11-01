use super::*;
use std::{
    path::Path,
    time::{Duration, Instant},
};

fn git(dir: &Path, args: &[&str]) -> String {
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
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}

fn commit_all(root: &Path, message: &str) -> String {
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", message]);
    git(root, &["rev-parse", "HEAD"])
}

fn write(root: &Path, name: &str, body: &str) {
    std::fs::write(root.join(name), body).unwrap();
}

fn repo() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    git(temp.path(), &["init", "-q", "-b", "main"]);
    temp
}

fn change(kind: ChangeKind, path: &str, old: Option<&str>) -> Change {
    Change {
        kind,
        path: path.to_owned(),
        old_path: old.map(str::to_owned),
    }
}

fn files(root: &Path, id: &str) -> Vec<Change> {
    changed_files(root.to_str().unwrap(), id, deadline(), &|| false).unwrap()
}

#[test]
fn parses_added_modified_deleted() {
    let got = parse_changes(b"A\0new.rs\0M\0src/lib.rs\0D\0gone.rs\0").unwrap();
    assert_eq!(
        got,
        vec![
            change(ChangeKind::Added, "new.rs", None),
            change(ChangeKind::Modified, "src/lib.rs", None),
            change(ChangeKind::Deleted, "gone.rs", None),
        ]
    );
}

#[test]
fn parses_renamed_and_copied_with_scores() {
    let got = parse_changes(b"R087\0old.rs\0new.rs\0C100\0a.rs\0b.rs\0").unwrap();
    assert_eq!(
        got,
        vec![
            change(ChangeKind::Renamed, "new.rs", Some("old.rs")),
            change(ChangeKind::Copied, "b.rs", Some("a.rs")),
        ]
    );
}

#[test]
fn parses_type_changed_and_unknown_status() {
    let got = parse_changes(b"T\0link\0X\0odd\0").unwrap();
    assert_eq!(
        got,
        vec![
            change(ChangeKind::TypeChanged, "link", None),
            change(ChangeKind::Other, "odd", None),
        ]
    );
}

#[test]
fn parses_paths_with_spaces_and_unicode() {
    let got = parse_changes("A\0my file.txt\0M\0dir/naïve ☃.rs\0".as_bytes()).unwrap();
    assert_eq!(
        got,
        vec![
            change(ChangeKind::Added, "my file.txt", None),
            change(ChangeKind::Modified, "dir/naïve ☃.rs", None),
        ]
    );
}

#[test]
fn empty_input_is_an_empty_list() {
    assert!(parse_changes(b"").unwrap().is_empty());
    // The stub also returns empty for non-empty input; make this one discriminate.
    assert_eq!(parse_changes(b"A\0x\0").unwrap().len(), 1);
}

#[test]
fn an_odd_field_count_is_malformed() {
    assert!(matches!(
        parse_changes(b"A\0x\0M\0"),
        Err(Error::Malformed { .. })
    ));
    assert!(matches!(
        parse_changes(b"R100\0only-one\0"),
        Err(Error::Malformed { .. })
    ));
}

#[test]
fn an_empty_status_is_malformed() {
    assert!(matches!(
        parse_changes(b"\0x\0"),
        Err(Error::Malformed { .. })
    ));
}

#[test]
fn oversize_input_is_rejected() {
    let big = vec![b'A'; MAX_INPUT + 1];
    assert!(matches!(parse_changes(&big), Err(Error::TooLarge { .. })));
}

#[test]
fn results_are_truncated_at_the_cap() {
    let mut input = Vec::new();
    for n in 0..MAX_CHANGES + 10 {
        input.extend_from_slice(format!("A\0f{n}\0").as_bytes());
    }
    let got = parse_changes(&input).unwrap();
    assert_eq!(got.len(), MAX_CHANGES);
    assert_eq!(got[MAX_CHANGES - 1].path, format!("f{}", MAX_CHANGES - 1));
}

#[test]
fn a_root_commit_lists_all_its_files_as_added() {
    let temp = repo();
    let root = temp.path();
    write(root, "a.txt", "a");
    write(root, "b.txt", "b");
    let id = commit_all(root, "root");
    let mut got = files(root, &id);
    got.sort_by(|x, y| x.path.cmp(&y.path));
    assert_eq!(
        got,
        vec![
            change(ChangeKind::Added, "a.txt", None),
            change(ChangeKind::Added, "b.txt", None),
        ]
    );
}

#[test]
fn a_normal_commit_lists_modified_added_and_deleted() {
    let temp = repo();
    let root = temp.path();
    write(root, "keep.txt", "1");
    write(root, "drop.txt", "1");
    commit_all(root, "base");
    write(root, "keep.txt", "2");
    write(root, "fresh.txt", "new");
    std::fs::remove_file(root.join("drop.txt")).unwrap();
    let id = commit_all(root, "change");
    let mut got = files(root, &id);
    got.sort_by(|x, y| x.path.cmp(&y.path));
    assert_eq!(
        got,
        vec![
            change(ChangeKind::Deleted, "drop.txt", None),
            change(ChangeKind::Added, "fresh.txt", None),
            change(ChangeKind::Modified, "keep.txt", None),
        ]
    );
}

#[test]
fn a_rename_reports_the_old_path() {
    let temp = repo();
    let root = temp.path();
    write(root, "old name.txt", &"line\n".repeat(50));
    commit_all(root, "base");
    git(root, &["mv", "old name.txt", "new name.txt"]);
    let id = commit_all(root, "rename");
    assert_eq!(
        files(root, &id),
        vec![change(
            ChangeKind::Renamed,
            "new name.txt",
            Some("old name.txt")
        )]
    );
}

#[test]
fn a_merge_commit_lists_only_changes_against_its_first_parent() {
    let temp = repo();
    let root = temp.path();
    write(root, "base.txt", "b");
    commit_all(root, "base");
    git(root, &["checkout", "-q", "-b", "feature"]);
    write(root, "side.txt", "s");
    commit_all(root, "side");
    git(root, &["checkout", "-q", "main"]);
    write(root, "main.txt", "m");
    commit_all(root, "main work");
    git(root, &["merge", "-q", "--no-ff", "-m", "merge", "feature"]);
    let id = git(root, &["rev-parse", "HEAD"]);
    assert_eq!(
        files(root, &id),
        vec![change(ChangeKind::Added, "side.txt", None)]
    );
}

#[test]
fn an_invalid_commit_id_is_rejected_before_git_runs() {
    let sixty_five = "a".repeat(65);
    for bad in ["--output=x", "ABCD1234", "abc", "", "abcg1234", &sixty_five] {
        // A path that is not a repository: if git ran, the error would be a
        // Git failure, not InvalidCommit.
        let result = changed_files("/nonexistent-herdr-path", bad, deadline(), &|| false);
        assert!(
            matches!(result, Err(crate::Error::Graph(Error::InvalidCommit))),
            "{bad:?}: {result:?}"
        );
    }
}

#[test]
fn a_cancelled_request_returns_an_error() {
    let temp = repo();
    let root = temp.path();
    write(root, "a.txt", "a");
    let id = commit_all(root, "root");
    let result = changed_files(root.to_str().unwrap(), &id, deadline(), &|| true);
    assert!(result.is_err(), "{result:?}");
}
