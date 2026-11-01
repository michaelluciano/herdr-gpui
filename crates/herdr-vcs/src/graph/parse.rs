//! The commit log of a checkout as typed commits, parsed strictly and within
//! fixed bounds: the output is untrusted text from a child process.

use super::error::{Error, Field};

/// Largest log output accepted, in bytes.
pub const MAX_INPUT: usize = 32 * 1024 * 1024;
/// Most commits accepted in one log.
pub const MAX_COMMITS: usize = 10_000;
/// Longest subject kept, in bytes; longer ones are cut on a character boundary.
const MAX_SUBJECT: usize = 512;
/// Longest author name kept, in bytes.
const MAX_AUTHOR: usize = 256;
/// A SHA-1 or SHA-256 object name.
const ID_LENGTHS: [usize; 2] = [40, 64];
const RECORD_END: u8 = 0x1e;

/// One commit of the log, newest first as Git listed it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Commit {
    /// Author name, cut to a bounded length.
    pub author: String,
    /// Full object name.
    pub id: String,
    /// Object names of the parents, in Git's order (first parent first).
    pub parents: Vec<String>,
    /// Branches, tags, and HEAD pointing at this commit.
    pub refs: Vec<Ref>,
    /// First line of the message, cut to a bounded length.
    pub subject: String,
    /// Author time in seconds since the Unix epoch.
    pub time: i64,
}

/// A name that points at a commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ref {
    /// HEAD, attached or detached.
    Head,
    /// `refs/heads/<name>`.
    LocalBranch(String),
    /// Any other ref, such as a stash or a note.
    Other,
    /// `refs/remotes/<remote>/<name>`.
    RemoteBranch { name: String, remote: String },
    /// `refs/tags/<name>`.
    Tag(String),
}

/// Parse `git log --format=%H%x00%P%x00%an%x00%at%x00%D%x00%s%x1e` output
/// decorated with full ref names. Rejects malformed or oversized output with a
/// typed error and never panics.
pub fn parse_log(output: &[u8]) -> Result<Vec<Commit>, Error> {
    if output.len() > MAX_INPUT {
        return Err(Error::TooLarge { max: MAX_INPUT });
    }
    let mut segments = output.split(|byte| *byte == RECORD_END);
    // `split` always yields one segment; what follows the last terminator
    // must be only the newline Git prints between records.
    let tail = segments.next_back().unwrap_or_default();
    if !tail.iter().all(u8::is_ascii_whitespace) {
        return Err(Error::Truncated);
    }
    let mut commits = Vec::new();
    for (index, record) in segments.enumerate() {
        if index >= MAX_COMMITS {
            return Err(Error::TooMany { max: MAX_COMMITS });
        }
        commits.push(parse_record(index, record)?);
    }
    Ok(commits)
}

fn parse_record(index: usize, record: &[u8]) -> Result<Commit, Error> {
    let malformed = |field| Error::Malformed { index, field };
    let record = record.trim_ascii_start();
    let fields: Vec<&[u8]> = record.split(|byte| *byte == 0).collect();
    let [id, parents, author, time, refs, subject] = fields[..] else {
        return Err(malformed(Field::Shape));
    };
    let id = object_name(id).ok_or_else(|| malformed(Field::Id))?;
    let parents = parents
        .split(u8::is_ascii_whitespace)
        .filter(|name| !name.is_empty())
        .map(|name| object_name(name).ok_or_else(|| malformed(Field::Parent)))
        .collect::<Result<Vec<_>, _>>()?;
    let time = std::str::from_utf8(time)
        .ok()
        .and_then(|time| time.parse().ok())
        .ok_or_else(|| malformed(Field::Time))?;
    Ok(Commit {
        author: bounded(author, MAX_AUTHOR),
        id,
        parents,
        refs: parse_refs(&String::from_utf8_lossy(refs)),
        subject: bounded(subject, MAX_SUBJECT),
        time,
    })
}

fn object_name(bytes: &[u8]) -> Option<String> {
    if !ID_LENGTHS.contains(&bytes.len()) || !bytes.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    Some(String::from_utf8_lossy(bytes).to_ascii_lowercase())
}

/// Lossy text cut to at most `max` bytes without splitting a character.
fn bounded(bytes: &[u8], max: usize) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut end = text.len().min(max);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_owned()
}

/// `%D` with full names: `HEAD -> refs/heads/x, refs/remotes/o/x, tag: refs/tags/v1`.
fn parse_refs(decoration: &str) -> Vec<Ref> {
    let mut refs = Vec::new();
    for item in decoration.split(", ").filter(|item| !item.is_empty()) {
        if let Some(target) = item.strip_prefix("HEAD -> ") {
            refs.push(Ref::Head);
            refs.push(parse_name(target));
        } else if item == "HEAD" {
            refs.push(Ref::Head);
        } else {
            refs.push(parse_name(item.strip_prefix("tag: ").unwrap_or(item)));
        }
    }
    refs
}

fn parse_name(full: &str) -> Ref {
    if let Some(name) = full.strip_prefix("refs/heads/").filter(|n| !n.is_empty()) {
        return Ref::LocalBranch(name.to_owned());
    }
    if let Some(name) = full.strip_prefix("refs/tags/").filter(|n| !n.is_empty()) {
        return Ref::Tag(name.to_owned());
    }
    if let Some((remote, name)) = full
        .strip_prefix("refs/remotes/")
        .and_then(|rest| rest.split_once('/'))
        .filter(|(remote, name)| !remote.is_empty() && !name.is_empty())
    {
        return Ref::RemoteBranch {
            name: name.to_owned(),
            remote: remote.to_owned(),
        };
    }
    Ref::Other
}
