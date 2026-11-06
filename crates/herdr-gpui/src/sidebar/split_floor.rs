//! The lowest the spaces/agents divider may sit: never above the workspace
//! list's own content, so dragging or mis-clicking it cannot hide workspaces.

use gpui::ScrollHandle;

const MIN_SPLIT: f32 = 0.1;
pub(super) const MAX_SPLIT: f32 = 0.9;

/// The smallest split ratio that keeps the whole workspace list visible.
/// `chrome` is the spaces section's pixels outside the scrolling list.
pub(super) fn split_floor(spaces_content: f32, chrome: f32, available: f32) -> f32 {
    if !available.is_finite() || available <= 0. {
        return MIN_SPLIT;
    }
    let ratio = (spaces_content + chrome) / available;
    if ratio.is_finite() {
        ratio.clamp(MIN_SPLIT, MAX_SPLIT)
    } else {
        MIN_SPLIT
    }
}

/// Height of everything the list holds, from the last frame's measurements.
pub(super) fn list_content_height(scroll: &ScrollHandle) -> f32 {
    let viewport = scroll.bounds();
    let overflow = scroll.max_offset().y;
    if overflow > gpui::px(0.) {
        return f32::from(viewport.size.height + overflow);
    }
    scroll
        .children_count()
        .checked_sub(1)
        .and_then(|last| scroll.bounds_for_item(last))
        .map_or(0., |item| f32::from(item.bottom() - viewport.origin.y))
}

/// The floor for a drag: the list's content plus the section chrome above it
/// (the spaces section starts at the sidebar's top, `origin_y`) and the row
/// `foot` pixels tall under it.
pub(super) fn drag_floor(scroll: &ScrollHandle, origin_y: f32, foot: f32, available: f32) -> f32 {
    let chrome = (f32::from(scroll.bounds().origin.y) - origin_y).max(0.) + foot;
    split_floor(list_content_height(scroll), chrome, available)
}

#[cfg(test)]
mod tests;
