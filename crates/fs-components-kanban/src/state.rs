//! The state of the board: selection, cursor, the card held, and the moves.
//!
//! Pattern: State + Command — one machine for every way to move a card (grab/hover/drop of a
//! frontend gesture, keys, move buttons, click on a column). It knows no Blinc and does not move
//! anything: a finished move comes out as a [`MoveRequest`] that the host answers.

use fs_components_widgets::Key;

use crate::model::Column;

/// A card taken up and where it is held.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Held {
    /// The card.
    pub key: String,
    /// The column it came from.
    pub from: String,
    /// The column it is over now.
    pub over: String,
}

/// A move for the host to carry out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveRequest {
    /// The card.
    pub key: String,
    /// Column it leaves.
    pub from: String,
    /// Column it goes to.
    pub to: String,
}

/// What a step came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Nothing happened.
    Ignored,
    /// The state changed; draw again.
    Changed,
    /// A card was chosen.
    Select(String),
    /// A move for the host.
    Move(MoveRequest),
    /// Refused here; the code is the tail of `kanban.refused.<code>`.
    Refused(&'static str),
}

/// Everything the board remembers.
#[derive(Clone, Debug, Default)]
pub struct BoardState {
    /// The chosen card.
    pub selected: Option<String>,
    /// The card under the keyboard cursor.
    pub cursor: Option<String>,
    /// The card held, if any.
    pub held: Option<Held>,
    /// Code of the last refusal, shown until the next step.
    pub notice: Option<String>,
}

/// Column and row of a card.
pub fn locate(cols: &[Column], key: &str) -> Option<(usize, usize)> {
    cols.iter()
        .enumerate()
        .find_map(|(c, col)| col.cards.iter().position(|k| k.key == key).map(|r| (c, r)))
}

impl BoardState {
    /// Choose a card.
    pub fn select(&mut self, key: &str) -> Step {
        self.selected = Some(key.to_owned());
        self.cursor = Some(key.to_owned());
        self.notice = None;
        Step::Select(key.to_owned())
    }

    /// Pick a card up (a gesture's start, or Space).
    pub fn grab(&mut self, cols: &[Column], can_move: bool, key: &str) -> Step {
        if !can_move {
            return self.refuse("locked");
        }
        let Some((c, _)) = locate(cols, key) else {
            return Step::Ignored;
        };
        self.held = Some(Held {
            key: key.to_owned(),
            from: cols[c].key.clone(),
            over: cols[c].key.clone(),
        });
        self.cursor = Some(key.to_owned());
        self.notice = None;
        Step::Changed
    }

    /// The held card is over `column`.
    pub fn hover(&mut self, cols: &[Column], column: &str) -> Step {
        match &mut self.held {
            Some(h) if cols.iter().any(|c| c.key == column) && h.over != column => {
                column.clone_into(&mut h.over);
                Step::Changed
            }
            _ => Step::Ignored,
        }
    }

    /// Put the held card down where it is; a move when the column differs.
    pub fn drop_held(&mut self) -> Step {
        match self.held.take() {
            Some(h) if h.from != h.over => Step::Move(MoveRequest {
                key: h.key,
                from: h.from,
                to: h.over,
            }),
            Some(_) => Step::Changed,
            None => Step::Ignored,
        }
    }

    /// Give the held card up.
    pub fn cancel(&mut self) -> Step {
        if self.held.take().is_some() {
            Step::Changed
        } else {
            Step::Ignored
        }
    }

    /// Move a card one column left or right (`dir` -1 or 1): the move buttons.
    pub fn step_over(&mut self, cols: &[Column], can_move: bool, key: &str, dir: i32) -> Step {
        if !can_move {
            return self.refuse("locked");
        }
        let Some((c, _)) = locate(cols, key) else {
            return Step::Ignored;
        };
        let Some(to) = c.checked_add_signed(dir as isize).and_then(|i| cols.get(i)) else {
            return Step::Ignored;
        };
        self.notice = None;
        Step::Move(MoveRequest {
            key: key.to_owned(),
            from: cols[c].key.clone(),
            to: to.key.clone(),
        })
    }

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
                Key::Activate | Key::Space => self.drop_held(),
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
            Some((c, r)) => {
                self.cursor = Some(cols[c].cards[r].key.clone());
                self.notice = None;
                Step::Changed
            }
            None => Step::Ignored,
        }
    }

    fn refuse(&mut self, code: &'static str) -> Step {
        self.notice = Some(code.to_owned());
        Step::Refused(code)
    }
}
