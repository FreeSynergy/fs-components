//! The two boundaries to the host: where items come from and who hears of choices.
//!
//! Pattern: Strategy (`TimelineSource`, `TimelineEvents`) with a Null object (`NoEvents`). The view
//! asks the source on every build, so the host stays the one owner of its data.

use crate::model::{Dependency, Item, Zoom};

/// Where items and dependencies come from.
pub trait TimelineSource: Send + Sync {
    /// The items, one row each, in order.
    fn items(&self) -> Vec<Item>;
    /// The dependencies between them (keys not in `items` are left out when drawing).
    fn dependencies(&self) -> Vec<Dependency>;
}

/// A fixed list.
#[derive(Clone, Debug, Default)]
pub struct VecSource {
    /// The items.
    pub items: Vec<Item>,
    /// The dependencies.
    pub dependencies: Vec<Dependency>,
}

impl TimelineSource for VecSource {
    fn items(&self) -> Vec<Item> {
        self.items.clone()
    }
    fn dependencies(&self) -> Vec<Dependency> {
        self.dependencies.clone()
    }
}

/// What the timeline tells the host. Every method has an empty default.
pub trait TimelineEvents: Send + Sync {
    /// The reader chose an item (click or Up/Down).
    fn select_item(&self, _key: &str) {}
    /// The reader changed the zoom.
    fn zoomed(&self, _zoom: Zoom) {}
}

/// Hears nothing.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoEvents;

impl TimelineEvents for NoEvents {}
