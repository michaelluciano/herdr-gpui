//! The changed files of the selected commit, in the detail pane.
use crate::{HerdrWindow, browser::Slot, graph_view::state::GraphState};
use gpui::{prelude::*, *};
use herdr_vcs::graph::{Change, ChangeKind};

/// Most file rows drawn; the rest are summed in a "+N more" line.
const MAX_FILE_ROWS: usize = 200;

fn letter(kind: ChangeKind) -> char {
    match kind {
        ChangeKind::Added => 'A',
        ChangeKind::Modified => 'M',
        ChangeKind::Deleted => 'D',
        ChangeKind::Renamed => 'R',
        ChangeKind::Copied => 'C',
        ChangeKind::TypeChanged => 'T',
        ChangeKind::Other => '?',
    }
}

fn file_text(change: &Change) -> String {
    let letter = letter(change.kind);
    match &change.old_path {
        Some(old) => format!("{letter} {old} \u{2192} {}", change.path),
        None => format!("{letter} {}", change.path),
    }
}

impl HerdrWindow {
    /// The file list of `commit`, or a loading line until it arrives.
    pub(super) fn graph_files(&self, slot: Slot, state: &GraphState, commit: &str) -> Div {
        let theme = &self.theme;
        let list = div().flex().flex_col().mt_1();
        let Some(files) = state.files.as_ref().filter(|files| files.commit == commit) else {
            return list.child(
                div()
                    .debug_selector(move || slot.selector("graph-files-loading"))
                    .text_color(rgb(theme.muted))
                    .child("Loading files"),
            );
        };
        let mut list = list;
        for (index, change) in files.changes.iter().take(MAX_FILE_ROWS).enumerate() {
            list = list.child(
                div()
                    .debug_selector(move || slot.selector(&format!("graph-file-{index}")))
                    .truncate()
                    .child(file_text(change)),
            );
        }
        let more = files.changes.len().saturating_sub(MAX_FILE_ROWS);
        if more > 0 {
            list = list.child(
                div()
                    .text_color(rgb(theme.muted))
                    .child(format!("+{more} more")),
            );
        }
        list
    }
}
