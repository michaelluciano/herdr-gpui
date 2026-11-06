//! Reordering the agents panel by dragging a row. Moving a pressed row a few
//! pixels lifts it; it follows the pointer and a line marks the gap it would
//! land in. Releasing freezes the new order with the panel's lock. The order
//! is local, so there is no daemon move and no slide preview.

use super::agent_lock::moved;
use crate::{
    HerdrWindow,
    reorder::{LIFT_DISTANCE, slot_for},
};
use gpui::{prelude::*, *};
use std::cell::{Cell, RefCell};

/// A press on an agent row that may become a reorder.
pub(crate) struct AgentDrag {
    agent: String,
    origin: Point<Pixels>,
    pointer: Point<Pixels>,
    lifted: bool,
    /// The gap the row would land in, `None` over its own place.
    slot: Option<usize>,
    /// How many rows the last frame painted.
    rows: usize,
    /// The pane ids the last frame painted, in row order: the rows the
    /// scroll's items measure, and the order a drop rewrites.
    painted: RefCell<Vec<String>>,
    /// How far the last frame painted the lifted row from its resting place.
    shift: Cell<f32>,
}

impl AgentDrag {
    /// The gap before painted row `slot` the pointer is over, measured from
    /// each row's resting span in the agents scroll, as workspace drags are.
    fn slot(&self, scroll: &ScrollHandle) -> Option<usize> {
        let painted = self.painted.borrow();
        let dragged = painted.iter().position(|id| *id == self.agent)?;
        let offset = scroll.offset().y;
        let spans: Vec<Option<(f32, f32)>> = (0..painted.len())
            .map(|row| {
                let bounds = scroll.bounds_for_item(row)?;
                let shift = if row == dragged { self.shift.get() } else { 0. };
                Some((
                    f32::from(bounds.top() + offset) - shift,
                    f32::from(bounds.bottom() + offset) - shift,
                ))
            })
            .collect();
        let (top, bottom) = spans[dragged]?;
        let lift = f32::from(self.pointer.y - self.origin.y);
        let slot = slot_for(&spans, dragged, (top + lift, bottom + lift));
        (slot != dragged && slot != dragged + 1).then_some(slot)
    }
}

impl HerdrWindow {
    /// Whether agent rows may be dragged: a plugin's agent view orders the
    /// panel itself, so the local order stands aside.
    pub(super) fn agent_drag_enabled(&self) -> bool {
        !self.listed_lives().any(|(_, live)| {
            live.snapshot
                .as_deref()
                .is_some_and(|snapshot| snapshot.agent_view_label.is_some())
        })
    }

    /// The row painted `position`-th for agent `id`, ready to be dragged:
    /// it records itself for measuring, arms a press, floats while lifted
    /// and marks the drop gap.
    pub(super) fn agent_drag_row(
        &self,
        row: Stateful<Div>,
        id: &str,
        position: usize,
        enabled: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if !enabled {
            return row.into_any_element();
        }
        let drag = self.agent_drag.as_ref();
        if let Some(drag) = drag {
            let mut painted = drag.painted.borrow_mut();
            painted.truncate(position);
            painted.push(id.to_owned());
        }
        let press_id = id.to_owned();
        let row = row.on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                if event.click_count == 1 {
                    this.press_agent(&press_id, event.position, cx);
                }
            }),
        );
        let Some(drag) = drag.filter(|drag| drag.lifted) else {
            return row.into_any_element();
        };
        let line = |at: Div| {
            at.absolute()
                .left_0()
                .right_0()
                .h(px(2.))
                .bg(rgb(self.theme.ink(self.theme.tokens().accent)))
        };
        // The gap after the last row has no row below it to carry the line.
        let row = row
            .when(drag.slot == Some(position), |row| {
                row.relative().child(line(div().top_0()))
            })
            .when(
                drag.slot == Some(position + 1) && position + 1 == drag.rows,
                |row| row.relative().child(line(div().bottom_0())),
            );
        if drag.agent != id {
            return row.into_any_element();
        }
        let offset = drag.pointer.y - drag.origin.y;
        drag.shift.set(f32::from(offset));
        // Painted last, over the rows it passes, while its layout slot keeps
        // the others where they are.
        deferred(row.relative().top(offset).cursor_grabbing())
            .with_priority(1)
            .into_any_element()
    }

    fn press_agent(&mut self, id: &str, position: Point<Pixels>, cx: &mut Context<Self>) {
        if self.menu.page.is_some()
            || self.workspace_drag.is_some()
            || self.device_drag.is_some()
            || !self.agent_drag_enabled()
        {
            return;
        }
        self.agent_drag = Some(AgentDrag {
            agent: id.to_owned(),
            origin: position,
            pointer: position,
            lifted: false,
            slot: None,
            rows: 0,
            painted: RefCell::default(),
            shift: Cell::new(0.),
        });
        cx.notify();
    }

    /// Follows the pointer. Returns whether the move belongs to the drag, so
    /// the terminal and hover never see it.
    pub(super) fn move_agent_drag(
        &mut self,
        position: Point<Pixels>,
        held: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let scroll = self.sidebar_scroll[1].clone();
        let Some(drag) = self.agent_drag.as_mut() else {
            return false;
        };
        // A release outside the window never reaches the sidebar.
        if !held {
            let lifted = drag.lifted;
            self.agent_drag = None;
            cx.notify();
            return lifted;
        }
        drag.pointer = position;
        let moved = position - drag.origin;
        if !drag.lifted && f32::from(moved.x.abs().max(moved.y.abs())) > LIFT_DISTANCE {
            drag.lifted = true;
            // The pointer is carrying a row, not asking for its menu.
            self.hover = None;
        }
        let Some(drag) = self.agent_drag.as_mut().filter(|drag| drag.lifted) else {
            return false;
        };
        drag.slot = drag.slot(&scroll);
        drag.rows = drag.painted.borrow().len();
        cx.notify();
        true
    }

    /// Ends the drag on release, freezing the new order if the row was
    /// lifted over a gap. Returns whether it had lifted, in which case the
    /// release is not a click.
    pub(super) fn release_agent_drag(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(drag) = self.agent_drag.take() else {
            return false;
        };
        if !drag.lifted {
            return false;
        }
        cx.notify();
        let order = drag
            .slot(&self.sidebar_scroll[1])
            .and_then(|slot| moved(&drag.painted.borrow(), &drag.agent, slot));
        if let Some(order) = order {
            self.agent_frozen = order;
            self.agent_lock = true;
            self.agent_lock_modified = true;
            self.save_chrome();
        }
        true
    }

    /// Drops a lifted row where it came from, and forgets a press that has
    /// not lifted. Returns whether one was lifted: only then is the key the
    /// drag's.
    pub(crate) fn cancel_agent_drag(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(drag) = self.agent_drag.take() else {
            return false;
        };
        if drag.lifted {
            cx.notify();
        }
        drag.lifted
    }
}
