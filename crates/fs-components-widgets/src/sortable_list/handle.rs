//! The handle of a sortable list: shared state a program and the view both hold.
//!
//! Pattern: Facade + Observer — one clonable handle over the [`Sorter`]; every call that changes
//! something answers with an [`Outcome`], tells the caller's callback, and draws the view again.
//! No method returns a lock guard: data comes out as copies.

use std::sync::{Arc, Mutex, PoisonError};

use super::bindings::{Bindings, Command};
use super::model::{Lane, Outcome, Place};
use super::state::Sorter;
use crate::keys::KeyEvent;
use crate::live::Live;

/// Called with what every step did.
pub type OnOutcome = Arc<dyn Fn(&Outcome) + Send + Sync>;

/// The words of the list.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// Shown in a lane without items.
    pub empty: String,
    /// Rows before the drawn ones; `{}` = how many.
    pub earlier: String,
    /// Rows after the drawn ones; `{}` = how many.
    pub later: String,
}

impl Default for Labels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            empty: s("Empty"),
            earlier: s("{} more above"),
            later: s("{} more below"),
        }
    }
}

struct Inner {
    sorter: Sorter,
    on_outcome: Option<OnOutcome>,
}

/// A sortable list. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct SortableList {
    shared: Arc<Mutex<Inner>>,
    pub(super) live: Live,
    pub(super) id: String,
    pub(super) bindings: Option<Bindings>,
    pub(super) labels: Labels,
    pub(super) max_rows: usize,
    pub(super) content: Option<super::view::Content>,
}

impl SortableList {
    /// A list over `lanes`. No keys are bound until [`SortableList::keys`] is called.
    pub fn new(lanes: Vec<Lane>) -> Self {
        Self {
            shared: Arc::new(Mutex::new(Inner {
                sorter: Sorter::new(lanes),
                on_outcome: None,
            })),
            live: Live::new(),
            id: "fs-sortable".into(),
            bindings: None,
            labels: Labels::default(),
            max_rows: 200,
            content: None,
        }
    }

    /// The element id of the list (rows are `<id>-<lane key>-<item key>`).
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Let the list take keys while it has the keyboard, by these bindings (e.g. the default).
    #[must_use]
    pub fn keys(mut self, bindings: Bindings) -> Self {
        self.bindings = Some(bindings);
        self
    }

    /// The words.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = labels;
        self
    }

    /// Draw at most `rows` rows per lane (the others are counted in a line).
    #[must_use]
    pub fn max_rows(mut self, rows: usize) -> Self {
        self.max_rows = rows;
        self
    }

    /// Hear of every step.
    #[must_use]
    pub fn on_outcome(self, handler: OnOutcome) -> Self {
        self.lock().on_outcome = Some(handler);
        self
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Run `f` on the sorter, tell the callback, draw again.
    fn act(&self, f: impl FnOnce(&mut Sorter) -> Outcome) -> Outcome {
        let (outcome, handler) = {
            let mut inner = self.lock();
            (f(&mut inner.sorter), inner.on_outcome.clone())
        };
        if let (true, Some(handler)) = (outcome != Outcome::Ignored, handler) {
            handler(&outcome);
        }
        self.live.refresh();
        outcome
    }

    /// Run `f` on the state without changing it (the view reads through this).
    pub(super) fn read<T>(&self, f: impl FnOnce(&Sorter) -> T) -> T {
        f(&self.lock().sorter)
    }

    /// A copy of the lanes as they are now.
    pub fn lanes(&self) -> Vec<Lane> {
        self.read(|s| s.lanes().to_vec())
    }

    /// Key of the item under the cursor.
    pub fn cursor_key(&self) -> Option<String> {
        self.read(|s| s.cursor_key().map(str::to_owned))
    }

    /// Key of the item held.
    pub fn grabbed(&self) -> Option<String> {
        self.read(|s| s.grabbed().map(str::to_owned))
    }

    /// Replace the data (it changed, or a move was not accepted).
    pub fn set_lanes(&self, lanes: Vec<Lane>) {
        self.lock().sorter.set_lanes(lanes);
        self.live.refresh();
    }

    /// Pick up an item (a gesture's start).
    pub fn grab(&self, key: &str) -> Outcome {
        self.act(|s| s.grab(key))
    }

    /// Carry the held item to a place (a gesture's move).
    pub fn hover(&self, to: &Place) -> Outcome {
        self.act(|s| s.hover(to))
    }

    /// Put the held item down (a gesture's end).
    pub fn drop_grabbed(&self) -> Outcome {
        self.act(Sorter::drop_grabbed)
    }

    /// Give up the hold.
    pub fn cancel(&self) -> Outcome {
        self.act(Sorter::cancel)
    }

    /// Move an item in one step.
    pub fn move_item(&self, key: &str, to: &Place) -> Outcome {
        self.act(|s| s.move_item(key, to))
    }

    /// Put the cursor on an item.
    pub fn set_cursor(&self, key: &str) -> Outcome {
        self.act(|s| s.set_cursor(key))
    }

    /// Do one step.
    pub fn command(&self, command: Command) -> Outcome {
        self.act(|s| s.command(command))
    }

    /// A key press: through the bindings if there are any ([`Outcome::Ignored`] otherwise).
    pub fn press(&self, event: &KeyEvent) -> Outcome {
        match self.bindings.as_ref().and_then(|b| b.command(event)) {
            Some(command) => self.command(command),
            None => Outcome::Ignored,
        }
    }
}
