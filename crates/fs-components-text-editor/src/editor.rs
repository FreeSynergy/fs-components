//! What a program needs from a text editor.
//!
//! Pattern: Strategy — programs depend on this trait; [`EditorModel`] (pure) and
//! [`crate::EditorView`] (with a window) implement it.

use crate::model::{Counts, EditCommand, EditorModel, Selection};

/// An editor of Markdown source.
pub trait TextEditor {
    /// The text.
    fn text(&self) -> String;
    /// Load a text: it counts as saved, the history is gone.
    fn set_text(&mut self, text: &str);
    /// The selection in characters.
    fn selection(&self) -> Selection;
    /// Select a range (cut to the text).
    fn select(&mut self, selection: Selection);
    /// Run a command on the selection; `true` if the text changed.
    fn apply(&mut self, command: &EditCommand) -> bool;
    /// The text was typed: take it as one undoable step; `true` if it differs.
    fn type_text(&mut self, text: &str, selection: Selection) -> bool;
    /// Take back the last step; `false` if none.
    fn undo(&mut self) -> bool;
    /// Do the last undone step again; `false` if none.
    fn redo(&mut self) -> bool;
    /// Is there something to take back?
    fn can_undo(&self) -> bool;
    /// Is there something to do again?
    fn can_redo(&self) -> bool;
    /// Differs the text from the saved one?
    fn is_dirty(&self) -> bool;
    /// The text as it is counts as saved.
    fn mark_saved(&mut self);
    /// Characters and words.
    fn counts(&self) -> Counts;
}

impl TextEditor for EditorModel {
    fn text(&self) -> String {
        EditorModel::text(self).to_owned()
    }
    fn set_text(&mut self, text: &str) {
        EditorModel::set_text(self, text);
    }
    fn selection(&self) -> Selection {
        EditorModel::selection(self)
    }
    fn select(&mut self, selection: Selection) {
        EditorModel::select(self, selection);
    }
    fn apply(&mut self, command: &EditCommand) -> bool {
        EditorModel::apply(self, command)
    }
    fn type_text(&mut self, text: &str, selection: Selection) -> bool {
        EditorModel::type_text(self, text, selection)
    }
    fn undo(&mut self) -> bool {
        EditorModel::undo(self)
    }
    fn redo(&mut self) -> bool {
        EditorModel::redo(self)
    }
    fn can_undo(&self) -> bool {
        EditorModel::can_undo(self)
    }
    fn can_redo(&self) -> bool {
        EditorModel::can_redo(self)
    }
    fn is_dirty(&self) -> bool {
        EditorModel::is_dirty(self)
    }
    fn mark_saved(&mut self) {
        EditorModel::mark_saved(self);
    }
    fn counts(&self) -> Counts {
        EditorModel::counts(self)
    }
}
