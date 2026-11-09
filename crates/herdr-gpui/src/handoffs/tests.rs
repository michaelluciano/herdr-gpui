//! Tests of the handoff list, its classification, its refresh, and its click target.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use std::path::{Path, PathBuf};

mod classify;
mod dismissed;
mod fast_refresh;
mod latest;
mod line;
mod list;
mod newest;
mod pinned;
mod poll;
mod target;

/// Real header shape of a session handoff, copied from the handoffs root.
pub(super) const CLAUDE_SETUP_HEADER: &str = "---\nid: S-2026-10-31-e8da4cd3\nparent: P-claude-setup\nprev: \nsession_id: e8da4cd3-a82f-45cf-83fc-da30f6606ae9\ncreated: 2026-10-31T13:41:42\nstatus: open\nctx_at_write: 0\nreason: manual\n---\n## Remaining work\nBody text.\n";

/// Real header shape of a claimed handoff, with a date only for `created`.
pub(super) const KB_HEADER_CLAIMED: &str = "---\nid: S37-lease-rerun-2026-10-31\nparent: kb\nprev: S36-lease-w4-t157-2026-10-31\nsession_id: pending (S37 claims with bin/bh build claim S37)\ncreated: 2026-10-31\nstatus: claimed\nctx_at_write: unknown\nreason: end\nclaimed_by: 184567ac-2533-4202-94f2-bf3b35b0b63e\n---\n# HEADLINE\n";

/// A header with `status: open` and the given id and created.
pub(super) fn open_header(id: &str, created: &str) -> String {
    format!(
        "---\nid: {id}\nparent: P-x\ncreated: {created}\nstatus: open\nreason: end\n---\nbody\n"
    )
}

/// A handoff as the loader reads it: the given id, project, created and
/// status, and no other front matter.
pub(super) fn fixture(id: &str, project: &str, created: &str, status: &str) -> Handoff {
    Handoff {
        path: PathBuf::from(format!("/handoffs/{project}/sessions/{id}.md")),
        project: project.into(),
        id: id.into(),
        created: created.into(),
        status: status.into(),
        session_id: String::new(),
        parent: String::new(),
        prev: String::new(),
        claimed_by: String::new(),
        modified: None,
    }
}

/// A unique temp directory removed on drop.
pub(super) struct Root(pub(super) PathBuf);

impl Root {
    pub(super) fn new(tag: &str) -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let path =
            std::env::temp_dir().join(format!("herdr-handoffs-{tag}-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    pub(super) fn path(&self) -> &Path {
        &self.0
    }

    /// Writes `<root>/<project>/sessions/<name>` and returns its path.
    pub(super) fn session(&self, project: &str, name: &str, bytes: impl AsRef<[u8]>) -> PathBuf {
        self.file(&format!("{project}/sessions/{name}"), bytes)
    }

    /// Writes `<root>/<relative>`, creating parents.
    pub(super) fn file(&self, relative: &str, bytes: impl AsRef<[u8]>) -> PathBuf {
        let path = self.0.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, bytes).unwrap();
        path
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).ok();
    }
}
