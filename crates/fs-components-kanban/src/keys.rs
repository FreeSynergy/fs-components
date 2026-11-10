//! Keys on the board: walk, pick up, choose the column, put down, give up.
//!
//! Pattern: State, part of `BoardState` (the transitions are in `state.rs`): a key is mapped to the
//! same transitions a gesture or a button uses.

use fs_components_widgets::Key;

use crate::model::Column;
use crate::state::{BoardState, Step, locate};

impl BoardState {
    /// A key press.
    pub fn key(&mut self, cols: &[Column], can_move: bool, key: Key) -> Step {
        if let Some(h) = self.held.clone() {
            let at = cols.iter().position(|c| c.key == h.over).unwrap_or(0);
            return match key {
                Key::Left | Key::Right => {
                    let to = if key == Key::Left {
                        at.checked_sub(1)
                    } else {
                        Some(at + 1)
                    };
                    to.and_then(|i| cols.get(i))
                        .map_or(Step::Ignored, |c| self.hover(cols, &c.key.clone()))
                }
                Key::Activate | Key::Space => self.drop_held(can_move),
                Key::Escape => self.cancel(),
                _ => Step::Ignored,
            };
        }
        match key {
            Key::Up | Key::Down | Key::Left | Key::Right => self.walk(cols, key),
            Key::Activate | Key::Space => match self.cursor.clone() {
                Some(k) => self.grab(cols, can_move, &k),
                None => Step::Ignored,
            },
            Key::Enter => self
                .cursor
                .clone()
                .map_or(Step::Ignored, |k| self.select(&k)),
            _ => Step::Ignored,
        }
    }

    fn walk(&mut self, cols: &[Column], key: Key) -> Step {
        let (c, r) = self
            .cursor
            .as_deref()
            .and_then(|k| locate(cols, k))
            .unwrap_or((usize::MAX, 0));
        let target = if c == usize::MAX {
            cols.iter()
                .position(|c| !c.cards.is_empty())
                .map(|c| (c, 0))
        } else {
            match key {
                Key::Up => Some((c, r.saturating_sub(1))),
                Key::Down => Some((c, (r + 1).min(cols[c].cards.len() - 1))),
                _ => {
                    let dir: isize = if key == Key::Left { -1 } else { 1 };
                    let mut i = c;
                    loop {
                        match i.checked_add_signed(dir).filter(|i| *i < cols.len()) {
                            Some(n) if cols[n].cards.is_empty() => i = n,
                            Some(n) => break Some((n, r.min(cols[n].cards.len() - 1))),
                            None => break None,
                        }
                    }
                }
            }
        };
        match target {
            Some((c, r)) if self.cursor.as_deref() != Some(cols[c].cards[r].key.as_str()) => {
                self.cursor = Some(cols[c].cards[r].key.clone());
                self.notice = None;
                Step::Changed
            }
            _ => Step::Ignored,
        }
    }
}
