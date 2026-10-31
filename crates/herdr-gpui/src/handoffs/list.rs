//! Lists the open handoff files of a handoffs root.

use std::{
    fs::File,
    io::Read as _,
    path::{Path, PathBuf},
};

/// The most handoffs kept.
pub(crate) const MAX_HANDOFFS: usize = 100;
/// The most bytes read from the head of one file.
pub(crate) const MAX_READ: usize = 4096;
/// The most characters kept of one front matter value.
pub(crate) const MAX_VALUE: usize = 120;

/// One open session handoff.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Handoff {
    pub(crate) path: PathBuf,
    /// The directory name above `sessions`.
    pub(crate) project: String,
    /// The front matter `id`, else the file stem.
    pub(crate) id: String,
    /// The front matter `created`, empty when absent.
    pub(crate) created: String,
}

/// The open `<root>/*/sessions/*.md` handoffs, newest first.
pub(crate) fn list_open(root: &Path) -> Vec<Handoff> {
    let mut found: Vec<Handoff> = entries(root, EntryKind::Dir)
        .into_iter()
        .flat_map(|project| {
            let name = project
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            // A symlinked `sessions` is not a directory to `file_type`.
            let sessions = project.join("sessions");
            let is_dir = std::fs::symlink_metadata(&sessions).is_ok_and(|m| m.is_dir());
            let files = if is_dir {
                entries(&sessions, EntryKind::File)
            } else {
                Vec::new()
            };
            files
                .into_iter()
                .filter(|file| file.extension().is_some_and(|ext| ext == "md"))
                .filter_map(|file| read_handoff(file, &name))
                .collect::<Vec<_>>()
        })
        .collect();
    found.sort_by(|a, b| b.created.cmp(&a.created).then_with(|| a.path.cmp(&b.path)));
    found.truncate(MAX_HANDOFFS);
    found
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EntryKind {
    Dir,
    File,
}

/// The children of `dir` of one kind; symlinks are neither.
fn entries(dir: &Path, kind: EntryKind) -> Vec<PathBuf> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    read.filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_ok_and(|ty| match kind {
                EntryKind::Dir => ty.is_dir(),
                EntryKind::File => ty.is_file(),
            })
        })
        .map(|entry| entry.path())
        .collect()
}

/// The handoff a file describes, `None` unless its front matter says open.
fn read_handoff(path: PathBuf, project: &str) -> Option<Handoff> {
    let head = read_head(&path)?;
    let mut lines = head.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let (mut id, mut created, mut status) = (None, None, None);
    let mut closed = false;
    for line in lines {
        if line.trim() == "---" {
            closed = true;
            break;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let slot = match key.trim() {
            "id" => &mut id,
            "created" => &mut created,
            "status" => &mut status,
            _ => continue,
        };
        slot.get_or_insert_with(|| value.trim().to_owned());
    }
    if !closed || status.as_deref() != Some("open") {
        return None;
    }
    let id = id
        .map(|id| cut(&id))
        .filter(|id| !id.is_empty())
        .or_else(|| path.file_stem().map(|s| cut(&s.to_string_lossy())))?;
    Some(Handoff {
        project: project.to_owned(),
        id,
        created: cut(&created.unwrap_or_default()),
        path,
    })
}

fn cut(value: &str) -> String {
    value.trim().chars().take(MAX_VALUE).collect()
}

/// The first [`MAX_READ`] bytes as text, cut back to a whole character.
fn read_head(path: &Path) -> Option<String> {
    let mut bytes = Vec::new();
    File::open(path)
        .ok()?
        .take(MAX_READ as u64)
        .read_to_end(&mut bytes)
        .ok()?;
    match String::from_utf8(bytes) {
        Ok(text) => Some(text),
        Err(err) if err.utf8_error().error_len().is_none() => {
            let valid = err.utf8_error().valid_up_to();
            let mut bytes = err.into_bytes();
            bytes.truncate(valid);
            String::from_utf8(bytes).ok()
        }
        Err(_) => None,
    }
}
