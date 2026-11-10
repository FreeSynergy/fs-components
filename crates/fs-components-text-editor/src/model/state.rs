//! The editor's state: text, selection, history, the saved text.
//!
//! Pattern: Command (the history is a stack of applied [`Edit`]s, undo and redo move them
//! between two stacks) with a Memento of the saved text for `is_dirty`.

use super::command::EditCommand;
use super::counts::Counts;
use super::edit::Edit;
use super::selection::Selection;

/// The most steps kept for undo.
pub const MAX_HISTORY: usize = 500;

/// A text being edited. No Blinc type here; tests need no window.
#[derive(Clone, Debug, Default)]
pub struct EditorModel {
    text: String,
    selection: Selection,
    saved: String,
    undo: Vec<Edit>,
    redo: Vec<Edit>,
}

impl EditorModel {
    /// A model holding `text`, which counts as saved.
    pub fn new(text: &str) -> Self {
        let mut model = Self::default();
        model.set_text(text);
        model
    }

    /// The text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The selection.
    pub fn selection(&self) -> Selection {
        self.selection
    }

    /// Select (cut to the text).
    pub fn select(&mut self, selection: Selection) {
        self.selection = selection.clamp(self.text.chars().count());
    }

    /// Load `text`: it counts as saved, the history is gone, the caret is at the end.
    pub fn set_text(&mut self, text: &str) {
        text.clone_into(&mut self.text);
        text.clone_into(&mut self.saved);
        self.undo.clear();
        self.redo.clear();
        self.selection = Selection::caret(text.chars().count());
    }

    fn record(&mut self, edit: Edit) {
        self.text = edit.apply(&self.text);
        self.selection = edit.after;
        self.redo.clear();
        self.undo.push(edit);
        if self.undo.len() > MAX_HISTORY {
            self.undo.remove(0);
        }
    }

    /// Run a command on the selection. `true` if the text changed.
    pub fn apply(&mut self, command: &EditCommand) -> bool {
        match command.apply(&self.text, self.selection) {
            Some(edit) => {
                self.record(edit);
                true
            }
            None => false,
        }
    }

    /// The text was changed from outside (typing): keep it as one step. `true` if it differs.
    pub fn type_text(&mut self, text: &str, selection: Selection) -> bool {
        let selection = selection.clamp(text.chars().count());
        let edit = Edit::between(&self.text, text, self.selection, selection);
        self.selection = selection;
        edit.is_some_and(|e| {
            self.record(e);
            true
        })
    }

    /// Take back the last step. `false` if there is none.
    pub fn undo(&mut self) -> bool {
        let Some(edit) = self.undo.pop() else {
            return false;
        };
        self.text = edit.invert().apply(&self.text);
        self.selection = edit.before;
        self.redo.push(edit);
        true
    }

    /// Do the last undone step again. `false` if there is none.
    pub fn redo(&mut self) -> bool {
        let Some(edit) = self.redo.pop() else {
            return false;
        };
        self.text = edit.apply(&self.text);
        self.selection = edit.after;
        self.undo.push(edit);
        true
    }

    /// Is there a step to take back?
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Is there a step to do again?
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Differs the text from the text at the start or the last `mark_saved`?
    pub fn is_dirty(&self) -> bool {
        self.text != self.saved
    }

    /// The text as it is now counts as saved.
    pub fn mark_saved(&mut self) {
        self.saved.clone_from(&self.text);
    }

    /// Characters and words.
    pub fn counts(&self) -> Counts {
        Counts::of(&self.text)
    }
}
