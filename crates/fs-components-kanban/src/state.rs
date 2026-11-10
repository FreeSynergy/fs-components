//! The state of the board: selection, cursor, the card held, and the moves.
//!
//! Pattern: State + Command — one machine for every way to move a card (grab/hover/drop of a
//! frontend gesture, keys, move buttons, click on a column). It knows no Blinc and does not move
//! anything: a finished move comes out as a [`MoveRequest`] that the host answers.

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
    /// `can_move` is asked again here: the host may have stopped supporting moves since the pick-up.
    pub fn drop_held(&mut self, can_move: bool) -> Step {
        match self.held.take() {
            Some(_) if !can_move => self.refuse("locked"),
            Some(h) if h.from != h.over => Step::Move(MoveRequest {
                key: h.key,
                from: h.from,
                to: h.over,
            }),
            Some(_) => Step::Changed,
            None => Step::Ignored,
        }
    }

    /// Hold over `column` and put down there, in one step.
    pub fn drop_at(&mut self, cols: &[Column], can_move: bool, column: &str) -> Step {
        if self.held.is_none() {
            return Step::Ignored;
        }
        self.hover(cols, column);
        self.drop_held(can_move)
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

    pub(crate) fn refuse(&mut self, code: &'static str) -> Step {
        self.notice = Some(code.to_owned());
        Step::Refused(code)
    }
}
