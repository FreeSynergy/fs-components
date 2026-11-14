//! The block editor as a building block behind the editor contract.
//!
//! Pattern: Strategy (one of the editors) — [`TextEditor`]: a document goes in, only operations of the model come out.
//!
//! The title block (a first `# ` heading that says the title) is the title field and no block of the editor; the edits of
//! the contract address the blocks of the document, so index 0 is the title field when there is one.

use fs_components_editor::{Doc, Edit, Op, TextEditor, diff};

use super::blocks::BlockEditor;

/// The title a document starts with: the text of its first block if that is a level-1 heading, else none.
fn title_of(doc: &Doc) -> String {
    doc.blocks
        .first()
        .and_then(|b| b.strip_prefix("# "))
        .map(|t| t.trim().to_owned())
        .unwrap_or_default()
}

impl TextEditor for BlockEditor {
    fn name(&self) -> &'static str {
        "block"
    }

    fn load(&mut self, doc: &Doc) {
        *self = Self::new(self.edit, &doc.to_markdown(), &title_of(doc));
        self.loaded = doc.clone();
    }

    fn document(&self) -> Doc {
        Doc::parse(&self.text())
    }

    fn edit(&mut self, edit: &Edit) {
        let shift = usize::from(self.in_text());
        match edit {
            Edit::Type { block, at, text } if *block < shift => {
                let line = format!("# {}", self.title_text());
                let at = (*at).min(line.len());
                let line = format!("{}{text}{}", &line[..at], &line[at..]);
                self.set_title(line.strip_prefix("# ").unwrap_or(&line));
            }
            Edit::Type { block, at, text } => self.blocks_type(block - shift, *at, text),
            // Enter in the title field only moves the keyboard to the first block.
            Edit::Enter { block, .. } if *block < shift => {}
            Edit::Enter { block, at } => self.blocks_enter(block - shift, *at),
        }
    }

    fn take_ops(&mut self) -> Vec<Op> {
        let now = self.document();
        let ops = diff(&self.loaded, &now);
        self.loaded = now;
        ops
    }
}
