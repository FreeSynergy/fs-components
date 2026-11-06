//! The data of a sortable list.
//!
//! Pattern: Value objects — neutral: an item has a key the caller understands and a label; the
//! widget does not know what the item is.

/// One item. Keys should be unique across all lanes.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Item {
    /// Identity given back in outcomes.
    pub key: String,
    /// What is shown.
    pub label: String,
    /// A locked item cannot be picked up (others may still move past it).
    pub locked: bool,
}

impl Item {
    /// An item.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            locked: false,
        }
    }

    /// Locked: cannot be moved.
    #[must_use]
    pub fn locked(mut self) -> Self {
        self.locked = true;
        self
    }
}

/// A list of items: one column of a board, or the only list.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lane {
    /// Identity given back in outcomes.
    pub key: String,
    /// Heading; empty for none.
    pub title: String,
    /// The items in order.
    pub items: Vec<Item>,
    /// Takes items from other lanes (its own items can always be reordered).
    pub open: bool,
}

impl Lane {
    /// An open lane.
    pub fn new(key: impl Into<String>, title: impl Into<String>, items: Vec<Item>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            items,
            open: true,
        }
    }

    /// A lane that does not take items from other lanes.
    #[must_use]
    pub fn closed(mut self) -> Self {
        self.open = false;
        self
    }
}

/// A place by position: lane number and index in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos {
    /// Which lane.
    pub lane: usize,
    /// Which item (or where an item would be put).
    pub index: usize,
}

/// A place by names: what outcomes and frontends use.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    /// Key of the lane.
    pub lane: String,
    /// Index in the lane.
    pub index: usize,
}

impl Place {
    /// A place.
    pub fn new(lane: impl Into<String>, index: usize) -> Self {
        Self {
            lane: lane.into(),
            index,
        }
    }
}

/// An item that changed place; the caller applies it to its own data.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Moved {
    /// Key of the item.
    pub item: String,
    /// Where it was before it was picked up.
    pub from: Place,
    /// Where it is now.
    pub to: Place,
}

impl Moved {
    /// A move of `item` from one place to another.
    pub fn new(item: impl Into<String>, from: Place, to: Place) -> Self {
        Self {
            item: item.into(),
            from,
            to,
        }
    }
}

/// Why a step did nothing.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// No item with that key.
    NoSuchItem,
    /// No lane with that key.
    NoSuchLane,
    /// The item is locked.
    Locked,
    /// The lane does not take items from other lanes.
    LaneClosed,
    /// An item is already held.
    Busy,
    /// Nothing is held.
    NothingHeld,
    /// Nowhere further in that direction.
    AtEdge,
    /// Nothing to act on (empty lists).
    Empty,
}

/// Where a choice came from: lets a consumer treat a click and a key differently.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// A click (or tap) on the row.
    Pointer,
    /// A key, through [`Command::Open`](super::Command::Open).
    Keyboard,
}

/// An item the person chose to open (as opposed to picking it up).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chosen {
    /// Key of the item.
    pub item: String,
    /// How it was chosen.
    pub source: Source,
}

impl Chosen {
    /// An item chosen from a source.
    pub fn new(item: impl Into<String>, source: Source) -> Self {
        Self {
            item: item.into(),
            source,
        }
    }
}

/// What a step did. Nothing happens without one of these.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The cursor moved (nothing changed in the data).
    Cursor,
    /// An item was chosen to be opened: by a click (the cursor is there now too) or by
    /// [`Command::Open`](super::Command::Open). The list does nothing more with it.
    Chosen(Chosen),
    /// An item is now held (key).
    Grabbed(String),
    /// The held item moved by one step; not final until dropped.
    Carried,
    /// The item was dropped at a new place.
    Moved(Moved),
    /// The item was dropped where it was picked up.
    Unchanged,
    /// The hold was given up and the item is back where it was.
    Cancelled,
    /// The step was not possible.
    Refused(Refusal),
    /// The key is not bound to anything.
    Ignored,
}
