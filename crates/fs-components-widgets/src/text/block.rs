//! Blocks: the parts a document is made of.
//!
//! Pattern: Composite — lists and quotes hold blocks again. Anything a producer cannot express
//! becomes [`Block::Plain`]: the text is shown as it is, never dropped.

use super::inline::Inline;

/// Alignment of a table column.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    /// Not specified (left).
    #[default]
    None,
    /// Left.
    Left,
    /// Centre.
    Center,
    /// Right.
    Right,
}

/// One block of a document.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// A heading of level 1 to 6.
    Heading {
        /// 1..=6.
        level: u8,
        /// The heading text.
        text: Inline,
    },
    /// A paragraph.
    Paragraph(Inline),
    /// A list; every item holds blocks.
    List {
        /// Numbered?
        ordered: bool,
        /// Number of the first item of a numbered list.
        start: u64,
        /// The items.
        items: Vec<ListItem>,
    },
    /// A quotation.
    Quote(Vec<Block>),
    /// Code, one string with line breaks.
    Code {
        /// Language hint (may be empty).
        lang: String,
        /// The code.
        text: String,
    },
    /// A table; the first row is the header.
    Table {
        /// Alignment per column.
        align: Vec<Align>,
        /// Rows of cells.
        rows: Vec<Vec<Inline>>,
    },
    /// A horizontal rule.
    Rule,
    /// Text that could not be mapped to anything else; shown as plain text.
    Plain(String),
}

/// One item of a list.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListItem {
    /// `Some(done)` for a task item.
    pub task: Option<bool>,
    /// The content.
    pub blocks: Vec<Block>,
}

impl ListItem {
    /// An item holding `blocks`.
    pub fn new(blocks: Vec<Block>) -> Self {
        Self { task: None, blocks }
    }

    /// A task item: `done` or not.
    #[must_use]
    pub fn task(mut self, done: bool) -> Self {
        self.task = Some(done);
        self
    }
}

/// A document: a title, blocks and an opaque revision.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Document {
    /// Title to show.
    pub title: String,
    /// The blocks in order.
    pub blocks: Vec<Block>,
    /// Opaque marker of this state of the document, if the source has one.
    pub revision: Option<String>,
}

impl Document {
    /// A document.
    pub fn new(title: impl Into<String>, blocks: Vec<Block>) -> Self {
        Self {
            title: title.into(),
            blocks,
            revision: None,
        }
    }

    /// The same document with a revision.
    #[must_use]
    pub fn with_revision(mut self, revision: impl Into<String>) -> Self {
        self.revision = Some(revision.into());
        self
    }
}
