//! The files one commit changed: one read-only Git child per call.

use super::{
    error::{Error, Field},
    parse::MAX_INPUT,
};
use std::time::Instant;

/// Most changes `parse_changes` returns; more are dropped by truncation.
pub const MAX_CHANGES: usize = 5000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Change {
    pub kind: ChangeKind,
    pub path: String,
    pub old_path: Option<String>,
}

/// Parses `git diff-tree --name-status -z` output: NUL-separated fields, a
/// status (`A`, `M`, `D`, `T`, `R<score>`, `C<score>`, anything else is
/// `Other`) then its path, or its old and new paths for `R` and `C`. Input over
/// `MAX_INPUT` is rejected; more than `MAX_CHANGES` changes are silently
/// truncated. Paths that are not UTF-8 are converted lossily, as the log
/// parser does.
pub fn parse_changes(output: &[u8]) -> Result<Vec<Change>, Error> {
    if output.len() > MAX_INPUT {
        return Err(Error::TooLarge { max: MAX_INPUT });
    }
    let body = output.strip_suffix(b"\0").unwrap_or(output);
    if body.is_empty() {
        return Ok(Vec::new());
    }
    let mut fields = body.split(|byte| *byte == 0);
    let mut changes = Vec::new();
    let mut index = 0;
    while changes.len() < MAX_CHANGES {
        let Some(status) = fields.next() else { break };
        let kind = match status.first() {
            None => return Err(shape(index)),
            Some(b'A') => ChangeKind::Added,
            Some(b'M') => ChangeKind::Modified,
            Some(b'D') => ChangeKind::Deleted,
            Some(b'T') => ChangeKind::TypeChanged,
            Some(b'R') => ChangeKind::Renamed,
            Some(b'C') => ChangeKind::Copied,
            Some(_) => ChangeKind::Other,
        };
        let first = fields.next().ok_or(shape(index))?;
        let (path, old_path) = if matches!(kind, ChangeKind::Renamed | ChangeKind::Copied) {
            let new = fields.next().ok_or(shape(index))?;
            (new, Some(text(first)))
        } else {
            (first, None)
        };
        changes.push(Change {
            kind,
            path: text(path),
            old_path,
        });
        index += 1;
    }
    Ok(changes)
}

fn shape(index: usize) -> Error {
    Error::Malformed {
        index,
        field: Field::Shape,
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The files `commit` changed against its first parent (all of them for a
/// root commit), with renames detected. Reads only; it blocks on Git child
/// processes sharing one `deadline`, so call it from a worker.
pub fn changed_files(
    checkout: &str,
    commit: &str,
    deadline: Instant,
    cancelled: &impl Fn() -> bool,
) -> crate::Result<Vec<Change>> {
    if !(4..=64).contains(&commit.len())
        || !commit
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(Error::InvalidCommit.into());
    }
    let git = |args: &[&str], what: &'static str| {
        crate::git::git_bytes(checkout, args, what, deadline, cancelled, MAX_INPUT)
    };
    let parents = git(
        &["rev-list", "--parents", "-n1", "--end-of-options", commit],
        "read the commit parents",
    )?;
    let first_line = parents
        .split(|byte| *byte == b'\n')
        .next()
        .unwrap_or_default();
    let has_parent = first_line.split(|byte| *byte == b' ').count() > 1;
    let first_parent = format!("{commit}^1");
    let output = if has_parent {
        git(
            &[
                "diff-tree",
                "-r",
                "-M",
                "--name-status",
                "-z",
                "--end-of-options",
                &first_parent,
                commit,
            ],
            "read the changed files",
        )?
    } else {
        git(
            &[
                "diff-tree",
                "--root",
                "--no-commit-id",
                "-r",
                "-M",
                "--name-status",
                "-z",
                "--end-of-options",
                commit,
            ],
            "read the changed files",
        )?
    };
    Ok(parse_changes(&output)?)
}
