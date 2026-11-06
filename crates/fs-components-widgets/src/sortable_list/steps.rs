//! The commands of a keyboard (or any other caller) applied to a [`Sorter`].
//!
//! Pattern: Command — one `match` from [`Command`] to the same methods a pointer uses.

use super::bindings::Command;
use super::model::{Chosen, Outcome, Pos, Refusal, Source};
use super::state::Sorter;

/// How far a page key goes.
const PAGE: usize = 10;

impl Sorter {
    /// Do one step.
    pub fn command(&mut self, command: Command) -> Outcome {
        match (self.grab.is_some(), command) {
            (true, Command::Toggle) => self.drop_grabbed(),
            (true, Command::Cancel) => self.cancel(),
            (false, Command::Cancel) => Outcome::Ignored,
            (_, Command::Open) => self.open(),
            (false, Command::Toggle) => match self.cursor.or_else(|| self.first_item()) {
                Some(pos) => self.grab_at(pos),
                None => Outcome::Refused(Refusal::Empty),
            },
            (_, Command::Left) => self.sideways(false),
            (_, Command::Right) => self.sideways(true),
            (_, c) => self.along(c),
        }
    }

    /// Choose the item under the cursor to be opened.
    pub fn open(&mut self) -> Outcome {
        match self.cursor_key() {
            Some(key) => Outcome::Chosen(Chosen::new(key, Source::Keyboard)),
            None => Outcome::Refused(Refusal::Empty),
        }
    }

    /// A click on the item with this key: the cursor goes there and the item is chosen.
    pub fn click(&mut self, key: &str) -> Outcome {
        match self.set_cursor(key) {
            Outcome::Cursor => Outcome::Chosen(Chosen::new(key, Source::Pointer)),
            other => other,
        }
    }

    /// Up, down, first, last, pages: within the lane of the cursor.
    fn along(&mut self, command: Command) -> Outcome {
        let Some(at) = self.cursor.or_else(|| self.first_item()) else {
            return Outcome::Refused(Refusal::Empty);
        };
        let len = self.lanes[at.lane].items.len();
        let last = len.saturating_sub(1);
        let target = match command {
            Command::Up => at.index.saturating_sub(1),
            Command::Down => (at.index + 1).min(last),
            Command::PageUp => at.index.saturating_sub(PAGE),
            Command::PageDown => (at.index + PAGE).min(last),
            Command::First => 0,
            _ => last,
        };
        if target == at.index && self.cursor.is_some() {
            return Outcome::Refused(Refusal::AtEdge);
        }
        self.go(Pos {
            lane: at.lane,
            index: target,
        })
    }

    /// To the next lane to the left or right that can be entered.
    fn sideways(&mut self, right: bool) -> Outcome {
        let Some(at) = self.cursor.or_else(|| self.first_item()) else {
            return Outcome::Refused(Refusal::Empty);
        };
        let held = self.grab.as_ref().map(|g| g.origin.lane.clone());
        let lanes = &self.lanes;
        let ok = |l: usize| match &held {
            Some(origin) => lanes[l].open || lanes[l].key == *origin,
            None => !lanes[l].items.is_empty(),
        };
        let found = if right {
            (at.lane + 1..lanes.len()).find(|&l| ok(l))
        } else {
            (0..at.lane).rev().find(|&l| ok(l))
        };
        let Some(lane) = found else {
            return Outcome::Refused(Refusal::AtEdge);
        };
        let len = self.lanes[lane].items.len();
        let index = if self.grab.is_some() {
            at.index.min(len)
        } else {
            at.index.min(len.saturating_sub(1))
        };
        self.go(Pos { lane, index })
    }

    /// The cursor, or the held item, goes to `pos`.
    fn go(&mut self, pos: Pos) -> Outcome {
        if self.grab.is_some() {
            return self.carry_to(pos.lane, pos.index);
        }
        self.cursor = Some(pos);
        Outcome::Cursor
    }
}
