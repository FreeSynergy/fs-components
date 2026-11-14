//! The source editor as a building block behind the editor contract: plain text (Markdown, code, configuration) in one text area.
//!
//! Pattern: Strategy (one of the editors) — [`TextEditor`]: a document goes in, only operations of the model come out. This
//! is the editor of the classic and the Confluence-like layout (`editor` draws its area); it never reformats: what is in the
//! area is what the person wrote.
//!
//! Limits: the area holds the whole text, so the operations are found by comparing the blocks before and after ([`diff`]);
//! a person typing in a block makes a replace of that block, a blank line inside a block makes a replace and an insert.

use blinc_layout::widgets::text_area::SharedTextAreaState;
use fs_components_editor::{Doc, Edit, Op, TextEditor, diff};
use fs_components_primitives::FieldData;

/// The text area of one edit and the document as it was when the edit began.
pub struct SourceEditor {
    area: SharedTextAreaState,
    _keep: FieldData,
    loaded: Doc,
}

impl Default for SourceEditor {
    fn default() -> Self {
        let keep = FieldData::new();
        Self {
            area: super::new_area("", &keep),
            _keep: keep,
            loaded: Doc::default(),
        }
    }
}

impl SourceEditor {
    /// A source editor with an empty text.
    pub fn new() -> Self {
        Self::default()
    }

    /// The text area (the layouts draw it).
    pub fn area(&self) -> &SharedTextAreaState {
        &self.area
    }

    fn text(&self) -> String {
        self.area
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .value()
    }

    fn set(&self, text: &str) {
        self.area
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_value(text);
    }

    /// Where block `i` starts in the text as it stands (blocks are one blank line apart).
    fn start_of(&self, i: usize) -> usize {
        Doc::parse(&self.text())
            .blocks
            .iter()
            .take(i)
            .map(|b| b.len() + 2)
            .sum()
    }
}

impl TextEditor for SourceEditor {
    fn name(&self) -> &'static str {
        "source"
    }

    fn load(&mut self, doc: &Doc) {
        self.set(&doc.to_markdown());
        self.loaded = doc.clone();
    }

    fn document(&self) -> Doc {
        Doc::parse(&self.text())
    }

    fn edit(&mut self, edit: &Edit) {
        let text = self.text();
        let (block, at, insert) = match edit {
            Edit::Type { block, at, text } => (*block, *at, text.as_str()),
            Edit::Enter { block, at } => (*block, *at, "\n"),
        };
        let place = (self.start_of(block) + at).min(text.len());
        self.set(&format!("{}{insert}{}", &text[..place], &text[place..]));
    }

    fn take_ops(&mut self) -> Vec<Op> {
        let now = self.document();
        let ops = diff(&self.loaded, &now);
        self.loaded = now;
        ops
    }
}
