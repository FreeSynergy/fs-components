//! The two boundaries to the host: where items come from and who hears of choices.
//!
//! Pattern: Strategy (`GridSource`, `GridEvents`) with a Null object (`NoEvents`). The grid asks
//! the source for the shown range on every build, so the host stays the one owner of its data; the
//! events are plain callbacks, the host decides what a choice means.

use crate::day::Day;
use crate::model::{GridItem, Nav, Slot};

/// Where the items of a range come from.
pub trait GridSource: Send + Sync {
    /// Items touching `first..=last` (more are fine, the grid cuts).
    fn items(&self, first: Day, last: Day) -> Vec<GridItem>;
}

/// A fixed list.
#[derive(Clone, Debug, Default)]
pub struct VecSource(pub Vec<GridItem>);

impl GridSource for VecSource {
    fn items(&self, first: Day, last: Day) -> Vec<GridItem> {
        self.0
            .iter()
            .filter(|i| i.start.day <= last && i.last_day() >= first)
            .cloned()
            .collect()
    }
}

/// What the grid tells the host. Every method has an empty default.
pub trait GridEvents: Send + Sync {
    /// The reader chose a slot (day cell, all-day row or time slot).
    fn select_slot(&self, _slot: Slot) {}
    /// The reader chose an item.
    fn select_item(&self, _key: &str) {}
    /// The shown range changed through a navigation message; `anchor` is the new day.
    fn navigated(&self, _nav: Nav, _anchor: Day) {}
}

/// Hears nothing.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoEvents;

impl GridEvents for NoEvents {}
