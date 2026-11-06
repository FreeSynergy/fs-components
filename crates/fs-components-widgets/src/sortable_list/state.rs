//! The state of a sortable list: lanes, a cursor, the item held.
//!
//! Pattern: State machine, pure — no Blinc, no clock, no thread. Everything a person can do is
//! a method here (the gestures of a frontend and the keys of [`Bindings`](super::Bindings) end up
//! in the same ones), and every method answers with an [`Outcome`].

use super::model::{Lane, Outcome, Place, Pos, Refusal};

/// An item picked up: where it came from and where it is now.
#[derive(Clone, Debug)]
pub(super) struct Grab {
    pub key: String,
    pub origin: Place,
    pub at: Pos,
}

/// Lanes of items with a cursor and at most one item held.
#[derive(Clone, Debug, Default)]
pub struct Sorter {
    pub(super) lanes: Vec<Lane>,
    pub(super) cursor: Option<Pos>,
    pub(super) grab: Option<Grab>,
}

impl Sorter {
    /// A sorter over `lanes`, the cursor on the first item.
    pub fn new(lanes: Vec<Lane>) -> Self {
        let mut s = Self {
            lanes,
            cursor: None,
            grab: None,
        };
        s.cursor = s.first_item();
        s
    }

    /// The lanes as they are now (also while an item is carried).
    pub fn lanes(&self) -> &[Lane] {
        &self.lanes
    }

    /// Replace the data (the caller's data changed, or it did not accept a move). The cursor and
    /// the held item stay with their keys when they still exist; otherwise they are let go.
    pub fn set_lanes(&mut self, lanes: Vec<Lane>) {
        let cursor_key = self.cursor.and_then(|p| self.key_at(p)).map(str::to_owned);
        self.lanes = lanes;
        if let Some(g) = self.grab.take()
            && let Some(at) = self.find(&g.key)
        {
            self.grab = Some(Grab { at, ..g });
        }
        self.cursor = cursor_key
            .and_then(|k| self.find(&k))
            .or_else(|| self.first_item());
    }

    /// The cursor, if there is an item.
    pub fn cursor(&self) -> Option<Pos> {
        self.cursor
    }

    /// Key of the item under the cursor.
    pub fn cursor_key(&self) -> Option<&str> {
        self.cursor.and_then(|p| self.key_at(p))
    }

    /// Key of the item held.
    pub fn grabbed(&self) -> Option<&str> {
        self.grab.as_ref().map(|g| g.key.as_str())
    }

    /// Where the item with this key is (the first, if keys are not unique).
    pub fn find(&self, key: &str) -> Option<Pos> {
        self.lanes.iter().enumerate().find_map(|(lane, l)| {
            l.items
                .iter()
                .position(|i| i.key == key)
                .map(|index| Pos { lane, index })
        })
    }

    /// The names of a position.
    pub fn place(&self, pos: Pos) -> Place {
        let lane = self.lanes.get(pos.lane).map_or("", |l| l.key.as_str());
        Place::new(lane, pos.index)
    }

    /// Put the cursor on the item with this key.
    pub fn set_cursor(&mut self, key: &str) -> Outcome {
        match self.find(key) {
            Some(pos) if self.grab.is_none() => {
                self.cursor = Some(pos);
                Outcome::Cursor
            }
            Some(_) => Outcome::Refused(Refusal::Busy),
            None => Outcome::Refused(Refusal::NoSuchItem),
        }
    }

    pub(super) fn key_at(&self, pos: Pos) -> Option<&str> {
        let item = self.lanes.get(pos.lane)?.items.get(pos.index)?;
        Some(item.key.as_str())
    }

    pub(super) fn first_item(&self) -> Option<Pos> {
        let lane = self.lanes.iter().position(|l| !l.items.is_empty())?;
        Some(Pos { lane, index: 0 })
    }

    pub(super) fn lane_index(&self, key: &str) -> Option<usize> {
        self.lanes.iter().position(|l| l.key == key)
    }
}
