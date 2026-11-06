//! Plain text of blocks (for search snippets and for tests).
//!
//! Pattern: Visitor-like fold — one function walks the composite and collects the text.

use super::block::Block;

impl Block {
    /// The text of this block without any formatting, blocks separated by line breaks.
    pub fn plain_text(&self) -> String {
        match self {
            Block::Heading { text, .. } | Block::Paragraph(text) => text.text.clone(),
            Block::List { items, .. } => items
                .iter()
                .flat_map(|i| i.blocks.iter().map(Block::plain_text))
                .collect::<Vec<_>>()
                .join("\n"),
            Block::Quote(blocks) => blocks
                .iter()
                .map(Block::plain_text)
                .collect::<Vec<_>>()
                .join("\n"),
            Block::Code { text, .. } | Block::Plain(text) => text.clone(),
            Block::Table { rows, .. } => rows
                .iter()
                .map(|r| {
                    r.iter()
                        .map(|c| c.text.as_str())
                        .collect::<Vec<_>>()
                        .join("\t")
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Block::Rule => String::new(),
        }
    }
}
