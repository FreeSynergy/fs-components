//! `TextEditor` for the view: the program uses the component through the trait.
//!
//! Pattern: Adapter — the trait over the model, with the text area kept in step.

use super::area;
use super::handle::EditorView;
use crate::editor::TextEditor;
use crate::model::{Counts, EditCommand, Selection};

impl TextEditor for EditorView {
    fn text(&self) -> String {
        self.lock().text().to_owned()
    }

    fn set_text(&mut self, text: &str) {
        self.lock().set_text(text);
        let now = self.lock().selection();
        area::write(&self.area, text, now);
        self.announce();
    }

    fn selection(&self) -> Selection {
        self.selection_now()
    }

    fn select(&mut self, selection: Selection) {
        self.lock().select(selection);
        let (text, now) = (self.lock().text().to_owned(), self.lock().selection());
        area::write(&self.area, &text, now);
    }

    fn apply(&mut self, command: &EditCommand) -> bool {
        self.run(|m| m.apply(command))
    }

    fn type_text(&mut self, text: &str, selection: Selection) -> bool {
        let changed = self.lock().type_text(text, selection);
        let now = self.lock().selection();
        area::write(&self.area, text, now);
        self.announce();
        changed
    }

    fn undo(&mut self) -> bool {
        self.run(crate::model::EditorModel::undo)
    }

    fn redo(&mut self) -> bool {
        self.run(crate::model::EditorModel::redo)
    }

    fn can_undo(&self) -> bool {
        self.lock().can_undo()
    }

    fn can_redo(&self) -> bool {
        self.lock().can_redo()
    }

    fn is_dirty(&self) -> bool {
        self.lock().is_dirty()
    }

    fn mark_saved(&mut self) {
        self.lock().mark_saved();
        self.announce();
    }

    fn counts(&self) -> Counts {
        self.lock().counts()
    }
}
