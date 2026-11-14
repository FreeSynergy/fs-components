//! The trait every editor building block stands behind.
//!
//! Pattern: Strategy — the program picks a building block; all of them take a [`Doc`] and give back [`Op`]s.

use crate::{Doc, Op};

/// A change a person makes, as the keyboard makes it. The conformance tests drive every editor with these, so they say
/// what happens to the text, not how a particular editor does it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edit {
    /// Type `text` into `block`, `at` bytes into it.
    Type {
        /// Which block.
        block: usize,
        /// Byte offset in the block.
        at: usize,
        /// What is typed (no line break).
        text: String,
    },
    /// Press Enter in `block`, `at` bytes into it.
    Enter {
        /// Which block.
        block: usize,
        /// Byte offset in the block.
        at: usize,
    },
}

/// An editor building block: a document goes in, only change operations of the same model come out.
///
/// Rules (the conformance tests check them): a document that is loaded and read back without an edit is the same
/// document and [`TextEditor::take_ops`] gives nothing; after edits, applying `take_ops` to the loaded document gives
/// exactly [`TextEditor::document`]; an editor changes data in no other way.
pub trait TextEditor {
    /// The fixed name of the building block (`source`, `block`, `rich`); also the base of its style names.
    fn name(&self) -> &'static str;

    /// Start editing `doc`.
    fn load(&mut self, doc: &Doc);

    /// The document as the editor holds it now.
    fn document(&self) -> Doc;

    /// Do one edit, as the keyboard would.
    fn edit(&mut self, edit: &Edit);

    /// The operations since `load` (or since the last call) that turn the document as it was then into the one now.
    fn take_ops(&mut self) -> Vec<Op>;
}
