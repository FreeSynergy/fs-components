//! The blocks of one edit as live elements: a column of text areas that grows when Enter makes a block.
//!
//! Pattern: Observer — the column depends on a signal that is counted up when the blocks change (Enter), and every block
//! depends on a signal of its own that is counted up when its width or its number of lines changed, so each is built
//! again with the room its text needs. All areas keep their state across the builds (cursor, focus, text).

use std::sync::{Arc, Mutex, PoisonError};

use blinc_core::reactive::{Signal, signal};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::stateful;
use blinc_layout::widgets::text_area::{SharedTextAreaState, TextPosition, focus_text_area};
use fs_components_primitives::FieldData;
use fs_components_widgets::SlashLabels;

use super::block_enter::{Enter, enter};
use super::block_text::{cursor_byte, set_text};

/// The words and the way text goes back to the editor's owner.
pub(super) type Report = Arc<dyn Fn(&str) + Send + Sync>;

/// Most rows one block asks for.
pub(super) const MOST_ROWS: usize = 400;

/// One block: its text area, the signals of that area kept alive, and its own "build me again" signal.
#[derive(Clone)]
pub(super) struct Slot {
    pub state: SharedTextAreaState,
    _keep: Arc<FieldData>,
    pub(super) grow: Signal<u64>,
    /// The text the area held after the last change (what "one line break more" is measured against).
    last: Arc<Mutex<String>>,
}

impl Slot {
    /// A block holding `text`.
    pub fn new(text: &str) -> Self {
        let keep = FieldData::new();
        let state = crate::view::new_area(text, &keep);
        let last = state.lock().unwrap_or_else(PoisonError::into_inner).value();
        Self {
            state,
            _keep: Arc::new(keep),
            grow: signal(0u64),
            last: Arc::new(Mutex::new(last)),
        }
    }
}

/// The blocks of an edit, shared by the editor and the elements it made.
pub(super) struct Blocks {
    slots: Mutex<Vec<Slot>>,
    structure: Signal<u64>,
    /// The open block menu, if any, and the signal that draws it again.
    pub(super) menu: Mutex<Option<super::block_menu::Open>>,
    pub(super) menu_signal: Signal<u64>,
    /// The words of the menu.
    pub(super) words: Mutex<SlashLabels>,
    /// Tells the owner the whole text (set when the column is made).
    announce_to: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

impl Blocks {
    /// The blocks for these texts.
    pub fn new(parts: &[String]) -> Arc<Self> {
        Arc::new(Self {
            slots: Mutex::new(parts.iter().map(|p| Slot::new(p)).collect()),
            structure: signal(0u64),
            menu: Mutex::new(None),
            menu_signal: signal(0u64),
            words: Mutex::new(SlashLabels::default()),
            announce_to: Mutex::new(None),
        })
    }

    fn list(&self) -> Vec<Slot> {
        self.slots
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The text of every block.
    pub fn values(&self) -> Vec<String> {
        self.list()
            .iter()
            .map(|s| {
                s.state
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .value()
            })
            .collect()
    }

    /// The area of block `i`.
    pub fn area(&self, i: usize) -> SharedTextAreaState {
        self.list()[i].state.clone()
    }

    /// How many blocks there are.
    pub fn len(&self) -> usize {
        self.list().len()
    }

    /// Which block has the keyboard.
    pub fn focused(&self) -> Option<usize> {
        self.list().iter().position(|s| {
            s.state
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .is_focused()
        })
    }

    /// Make a block of the text after the cursor of block `i` (or put the line break where the area left it).
    pub(super) fn on_changed(&self, state: &SharedTextAreaState) {
        let before = self
            .list()
            .iter()
            .find(|s| Arc::ptr_eq(&s.state, state))
            .map(|s| {
                s.last
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .clone()
            })
            .unwrap_or_default();
        self.changed_from(state, &before);
        self.remember(state);
    }

    /// Keep the text the area holds now as the one the next change is measured against.
    pub(super) fn remember(&self, state: &SharedTextAreaState) {
        let now = state.lock().unwrap_or_else(PoisonError::into_inner).value();
        if let Some(s) = self.list().iter().find(|s| Arc::ptr_eq(&s.state, state)) {
            *s.last.lock().unwrap_or_else(PoisonError::into_inner) = now;
        }
    }

    fn changed_from(&self, state: &SharedTextAreaState, before: &str) {
        let Some(i) = self
            .list()
            .iter()
            .position(|s| Arc::ptr_eq(&s.state, state))
        else {
            return;
        };
        let (now, cursor) = {
            let d = state.lock().unwrap_or_else(PoisonError::into_inner);
            (d.value(), cursor_byte(&d.lines, d.cursor))
        };
        // Enter is exactly one line break more, at the cursor; anything else is typing, pasting or deleting.
        let is_enter = cursor > 0
            && now.is_char_boundary(cursor - 1)
            && now.as_bytes().get(cursor - 1) == Some(&b'\n')
            && format!("{}{}", &now[..cursor - 1], &now[cursor..]) == before;
        if is_enter && self.marked(state).is_some() {
            // Enter in an open menu chooses the marked type; the line break the area made goes with the "/" text.
            if let Some(kind) = self.marked(state) {
                self.choose(state, kind);
            }
            return;
        }
        if is_enter {
            match enter(&now, cursor) {
                Enter::Keep => {}
                Enter::Carry { text, at } => {
                    set_text(state, &text, Some(at));
                }
                Enter::Split { head, tail } => {
                    set_text(state, &head, None);
                    self.insert_after(i, &tail);
                }
            }
        }
        let shown = state.lock().unwrap_or_else(PoisonError::into_inner).value();
        self.refresh_menu(state, &shown);
    }

    /// A new block holding `text` after block `i`; it gets the keyboard, the cursor at its start.
    fn insert_after(&self, i: usize, text: &str) {
        let slot = Slot::new(text);
        slot.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .cursor = TextPosition::new(0, 0);
        self.slots
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(i + 1, slot.clone());
        focus_text_area(&slot.state);
        self.structure.update(|v| v.wrapping_add(1));
    }

    /// A new block holding `text` after the block of `state`.
    pub(super) fn insert_after_state(&self, state: &SharedTextAreaState, text: &str) {
        if let Some(i) = self
            .list()
            .iter()
            .position(|s| Arc::ptr_eq(&s.state, state))
        {
            self.insert_after(i, text);
        }
    }

    /// Tell the owner the whole text now.
    pub(super) fn announce(&self) {
        let to = self
            .announce_to
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if let Some(to) = to {
            to();
        }
    }

    /// Press Enter in block `i`, `at` bytes into it: what the keyboard does, without the keyboard (the editor contract's
    /// edits go through here, so the tests of the contract and the window share the one code path).
    pub fn enter_at(&self, i: usize, at: usize) {
        let state = self.area(i);
        let before = state.lock().unwrap_or_else(PoisonError::into_inner).value();
        let at = at.min(before.len());
        let now = format!("{}\n{}", &before[..at], &before[at..]);
        set_text(&state, &now, Some(at + 1));
        self.changed_from(&state, &before);
        self.remember(&state);
    }

    /// Type `text` (no line break) into block `i`, `at` bytes into it.
    pub fn type_at(&self, i: usize, at: usize, text: &str) {
        let state = self.area(i);
        let before = state.lock().unwrap_or_else(PoisonError::into_inner).value();
        let at = at.min(before.len());
        let now = format!("{}{text}{}", &before[..at], &before[at..]);
        set_text(&state, &now, Some(at + text.len()));
        self.changed_from(&state, &before);
        self.remember(&state);
    }

    /// The column of blocks.
    pub fn view(
        self: &Arc<Self>,
        hint: &str,
        words: &SlashLabels,
        report: &Report,
        assemble: Arc<dyn Fn() -> String + Send + Sync>,
    ) -> Div {
        *self.words.lock().unwrap_or_else(PoisonError::into_inner) = words.clone();
        let (blocks, hint, report) = (Arc::clone(self), hint.to_owned(), Arc::clone(report));
        {
            let (report, assemble) = (Arc::clone(&report), Arc::clone(&assemble));
            *self
                .announce_to
                .lock()
                .unwrap_or_else(PoisonError::into_inner) =
                Some(Arc::new(move || report(&assemble())));
        }
        let keys = Arc::clone(self);
        let column = stateful::<NoState>()
            .deps([self.structure.id()])
            .on_state(move |_| {
                let mut column = div().flex_col().w_full().gap(1.0);
                for (i, slot) in blocks.list().into_iter().enumerate() {
                    column = column.child(super::block_row::block(
                        &blocks, i, &slot, &hint, &report, &assemble,
                    ));
                }
                column
            });
        div()
            .class("fs-wk-blocks")
            .w_full()
            .on_key_down(move |e| keys.menu_key(e.key_code))
            .child(column)
    }
}
