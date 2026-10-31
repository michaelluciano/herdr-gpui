//! What a click on a handoff row opens.

use super::list::Handoff;
use crate::editor::EditorTarget;

/// The code tab target of a handoff row: the file, at its top.
pub(crate) fn target_for(handoff: &Handoff) -> EditorTarget {
    EditorTarget {
        path: handoff.path.clone(),
        line: None,
    }
}
