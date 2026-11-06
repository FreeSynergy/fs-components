//! Blocks: the parts a document is made of.
//!
//! Pattern: Composite — lists and quotes hold blocks again. Anything a producer cannot express
//! becomes [`Block::Plain`]: the text is shown as it is, never dropped.

use super::inline::Inline;

/// Alignment of a table column.
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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListItem {
    /// `Some(done)` for a task item.
    pub task: Option<bool>,
    /// The content.
    pub blocks: Vec<Block>,
}

/// A document: a title and blocks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Document {
    /// Title to show.
    pub title: String,
    /// The blocks in order.
    pub blocks: Vec<Block>,
}
