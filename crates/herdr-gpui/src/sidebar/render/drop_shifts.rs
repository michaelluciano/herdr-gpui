//! The shift each unit takes while a workspace drag previews its drop.

use crate::{
    HerdrWindow,
    sidebar::{
        line_height,
        reorder::{MoveBlock, Plan, WorkspaceDrag},
    },
};
use herdr_client::protocol::ClientShellWorkspace;

/// Visible rows as `(index, child, group)`.
type Entries = [(usize, bool, Option<String>)];

impl HerdrWindow {
    /// Each unit's shift for the drop in preview, from the heights the last
    /// frame laid out: shifting moves rows, never resizes them. Also leaves the
    /// pointer handler's drop requests and the dragged unit in `drop`.
    pub(super) fn drop_shifts(
        &self,
        plan: Option<&Plan>,
        workspaces: &[ClientShellWorkspace],
        (entries, base): (&Entries, usize),
        (drag, spaces_custom): (Option<&WorkspaceDrag>, bool),
        (drop_requests, drop_dragged): (&mut Vec<Option<MoveBlock>>, &mut usize),
    ) -> Vec<f32> {
        let font = &self.config.sidebar;
        plan.map_or_else(Vec::new, |plan| {
            *drop_requests = (0..=plan.len())
                .map(|slot| plan.request(workspaces, slot))
                .collect();
            *drop_dragged = plan.dragged();
            let mut heights = vec![0.; plan.len()];
            for (position, entry) in entries.iter().enumerate() {
                if let (Some(unit), Some(bounds)) = (
                    plan.unit_of(entry.0),
                    self.sidebar_scroll[0].bounds_for_item(base + position),
                ) {
                    heights[unit] += f32::from(bounds.size.height);
                    if spaces_custom && !entry.1 {
                        heights[unit] += f32::from(self.config.sidebar_layout.spaces.row_gap)
                            * line_height(font);
                    }
                }
            }
            let slot = drag
                .and_then(|drag| drag.target.as_ref())
                .map_or(plan.dragged(), |target| target.slot);
            crate::reorder::preview(&heights, plan.dragged(), slot)
        })
    }
}
