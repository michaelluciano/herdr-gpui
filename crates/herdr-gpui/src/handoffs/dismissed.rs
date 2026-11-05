//! The handoffs the operator removed from the list. Saved per machine as a
//! JSON array of absolute file paths in the app's state directory; the
//! handoff files themselves are never touched.

use crate::state_file;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

const FILE: &str = "handoff-dismissed.json";
const MAX_FILE_BYTES: u64 = 512 * 1024;
const MAX_ENTRIES: usize = 4096;

/// Where the set is saved, `None` without a state directory.
pub(crate) fn file() -> Option<PathBuf> {
    crate::preferences::state_dir().map(|dir| dir.join(FILE))
}

/// The saved set; empty when the file is missing or damaged.
pub(crate) fn load(path: &Path) -> HashSet<PathBuf> {
    let read = state_file::read(path, MAX_FILE_BYTES).and_then(|bytes| {
        bytes.map_or(Ok(Vec::new()), |bytes| {
            serde_json::from_slice::<Vec<String>>(&bytes).map_err(Into::into)
        })
    });
    match read {
        Ok(paths) => paths
            .into_iter()
            .take(MAX_ENTRIES)
            .map(PathBuf::from)
            .collect(),
        Err(error) => {
            tracing::warn!(%error, "Cannot restore dismissed handoffs");
            HashSet::new()
        }
    }
}

/// The file's content for `set`, sorted so the file is stable.
pub(crate) fn to_saved(set: &HashSet<PathBuf>) -> Vec<String> {
    let mut paths: Vec<String> = set
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    paths.sort();
    paths.truncate(MAX_ENTRIES);
    paths
}
