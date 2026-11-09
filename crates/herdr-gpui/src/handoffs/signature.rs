//! A cheap fingerprint of the handoffs root, so a new, removed or edited
//! handoff is noticed without listing and parsing every file.

use super::list::{EntryKind, entries};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash as _, Hasher as _},
    path::Path,
};

/// Fingerprints what `list_all` reads: the root, each project directory, its
/// `LATEST.md`, and its `sessions` and `history` directories and their files,
/// by path, size and modification time. Blocking: run it off the UI thread.
pub(super) fn signature(root: &Path) -> u64 {
    let mut total = stamp(root);
    for project in entries(root, EntryKind::Dir) {
        total = total.wrapping_add(stamp(&project));
        total = total.wrapping_add(stamp(&project.join("LATEST.md")));
        for dir in ["sessions", "history"] {
            let dir = project.join(dir);
            total = total.wrapping_add(stamp(&dir));
            for file in entries(&dir, EntryKind::File) {
                total = total.wrapping_add(stamp(&file));
            }
        }
    }
    total
}

/// One path's own hash. Entries are summed, so the directory's listing order
/// does not matter.
fn stamp(path: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        meta.len().hash(&mut hasher);
        meta.modified().ok().hash(&mut hasher);
    }
    hasher.finish()
}
