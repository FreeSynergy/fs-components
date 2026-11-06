//! Picking up, carrying and putting down.
//!
//! Pattern: State transitions of [`Sorter`]: free → held → (dropped | cancelled). The item is
//! moved in the lanes at once while it is held, so a view always shows where it would land.

use super::model::{Moved, Outcome, Place, Pos, Refusal};
use super::state::{Grab, Sorter};

impl Sorter {
    /// Pick up the item with this key.
    pub fn grab(&mut self, key: &str) -> Outcome {
        match self.find(key) {
            Some(pos) => self.grab_at(pos),
            None => Outcome::Refused(Refusal::NoSuchItem),
        }
    }

    /// Pick up the item at a position.
    pub fn grab_at(&mut self, pos: Pos) -> Outcome {
        if self.grab.is_some() {
            return Outcome::Refused(Refusal::Busy);
        }
        let Some(item) = self
            .lanes
            .get(pos.lane)
            .and_then(|l| l.items.get(pos.index))
        else {
            return Outcome::Refused(Refusal::NoSuchItem);
        };
        if item.locked {
            return Outcome::Refused(Refusal::Locked);
        }
        let key = item.key.clone();
        self.grab = Some(Grab {
            key: key.clone(),
            origin: self.place(pos),
            at: pos,
        });
        self.cursor = Some(pos);
        Outcome::Grabbed(key)
    }

    /// Carry the held item to a place (the pointer is over it). `to.index` counts the lane
    /// without the held item; too large means the end.
    pub fn hover(&mut self, to: &Place) -> Outcome {
        let Some(grab) = &self.grab else {
            return Outcome::Refused(Refusal::NothingHeld);
        };
        let Some(lane) = self.lane_index(&to.lane) else {
            return Outcome::Refused(Refusal::NoSuchLane);
        };
        if !self.lanes[lane].open && self.lanes[lane].key != grab.origin.lane {
            return Outcome::Refused(Refusal::LaneClosed);
        }
        self.carry_to(lane, to.index)
    }

    /// Put the held item down where it is.
    pub fn drop_grabbed(&mut self) -> Outcome {
        let Some(grab) = self.grab.take() else {
            return Outcome::Refused(Refusal::NothingHeld);
        };
        let to = self.place(grab.at);
        if to == grab.origin {
            return Outcome::Unchanged;
        }
        Outcome::Moved(Moved {
            item: grab.key,
            from: grab.origin,
            to,
        })
    }

    /// Give up: the item goes back to where it was picked up.
    pub fn cancel(&mut self) -> Outcome {
        let Some(grab) = self.grab.clone() else {
            return Outcome::Refused(Refusal::NothingHeld);
        };
        if let Some(lane) = self.lane_index(&grab.origin.lane) {
            self.carry_to(lane, grab.origin.index);
        }
        self.grab = None;
        Outcome::Cancelled
    }

    /// Move an item in one step (a frontend that decides by itself where it goes).
    pub fn move_item(&mut self, key: &str, to: &Place) -> Outcome {
        let picked = self.grab(key);
        if !matches!(picked, Outcome::Grabbed(_)) {
            return picked;
        }
        match self.hover(to) {
            Outcome::Carried => self.drop_grabbed(),
            refused => {
                self.cancel();
                refused
            }
        }
    }

    /// Move the held item to `index` of lane `lane` (both exist).
    pub(super) fn carry_to(&mut self, lane: usize, index: usize) -> Outcome {
        let Some(at) = self.grab.as_ref().map(|g| g.at) else {
            return Outcome::Refused(Refusal::NothingHeld);
        };
        let item = self.lanes[at.lane].items.remove(at.index);
        let index = index.min(self.lanes[lane].items.len());
        self.lanes[lane].items.insert(index, item);
        let now = Pos { lane, index };
        if let Some(g) = &mut self.grab {
            g.at = now;
        }
        self.cursor = Some(now);
        Outcome::Carried
    }
}
