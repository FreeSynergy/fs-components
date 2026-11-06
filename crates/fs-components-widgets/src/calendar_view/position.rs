//! Where the axis and the agenda are scrolled to.
//!
//! Pattern: Memento — a handle the owner keeps, so a rebuild keeps the place: the axis starts at the
//! hour asked for the first time and then stays where the person left it; a list goes back to the
//! top when it shows another span. Nothing here hands out a lock.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

use super::day::Day;
use crate::list_view::ListPosition;

/// No day yet: the value of the atomic below.
const NO_DAY: i64 = i64::MIN;

/// The scroll positions of one calendar view. Cheap to clone; clones share the positions.
#[derive(Clone)]
pub struct CalendarPosition {
    axis: ListPosition,
    list: ListPosition,
    started: Arc<AtomicBool>,
    list_day: Arc<AtomicI64>,
}

impl Default for CalendarPosition {
    fn default() -> Self {
        Self {
            axis: ListPosition::new(),
            list: ListPosition::new(),
            started: Arc::new(AtomicBool::new(false)),
            list_day: Arc::new(AtomicI64::new(NO_DAY)),
        }
    }
}

impl CalendarPosition {
    /// At the top.
    pub fn new() -> Self {
        Self::default()
    }

    /// The axis, put at `offset` pixels the first time.
    pub(super) fn axis(&self, offset: Option<f32>) -> &ListPosition {
        if let Some(offset) = offset
            && !self.started.swap(true, Ordering::Relaxed)
        {
            self.axis.apply_offset(offset);
        }
        &self.axis
    }

    /// The list, at the top if it showed a span from another day before.
    pub(super) fn list(&self, first: Day) -> &ListPosition {
        if self.list_day.swap(first.number(), Ordering::Relaxed) != first.number() {
            self.list.apply_offset(0.0);
        }
        &self.list
    }

    /// How far the list is scrolled now, in pixels from the top.
    pub fn list_offset(&self) -> f32 {
        self.list.offset()
    }

    /// How far the axis is scrolled now, in pixels from the top.
    pub fn axis_offset(&self) -> f32 {
        self.axis.offset()
    }
}
