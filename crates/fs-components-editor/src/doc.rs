//! The document the editors work on: blocks of Markdown source.
//!
//! Pattern: none — plain data. A block is a run of lines between blank lines; a fenced code block is one block whatever
//! it holds. Text goes in and out in one canonical form: one blank line between blocks, one line break at the end.

/// A document: its blocks, each the Markdown source of one block, without the blank lines around it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Doc {
    /// The blocks in order.
    pub blocks: Vec<String>,
}

impl Doc {
    /// A document of these blocks.
    #[must_use]
    pub fn new(blocks: Vec<String>) -> Self {
        Self { blocks }
    }

    /// Cut `text` into blocks at blank lines; a fenced code block is one block whatever it contains.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut out: Vec<String> = Vec::new();
        let mut current: Vec<&str> = Vec::new();
        let mut fence: Option<&str> = None;
        for line in text.lines() {
            let marker = line.trim_start();
            let tick = ["```", "~~~"].into_iter().find(|f| marker.starts_with(f));
            match (fence, tick) {
                (None, Some(f)) => fence = Some(f),
                (Some(f), Some(t)) if f == t => fence = None,
                _ => {}
            }
            if fence.is_none() && line.trim().is_empty() && tick.is_none() {
                if !current.is_empty() {
                    out.push(current.join("\n"));
                    current.clear();
                }
            } else {
                current.push(line);
            }
        }
        if !current.is_empty() {
            out.push(current.join("\n"));
        }
        Self { blocks: out }
    }

    /// The blocks put together again: one blank line between, a line break at the end (none for no blocks).
    #[must_use]
    pub fn to_markdown(&self) -> String {
        if self.blocks.is_empty() {
            return String::new();
        }
        let mut text = self.blocks.join("\n\n");
        text.push('\n');
        text
    }

    /// Is `text` in the canonical form, so that it goes through [`Doc::parse`] and [`Doc::to_markdown`] unchanged?
    #[must_use]
    pub fn is_canonical(text: &str) -> bool {
        Self::parse(text).to_markdown() == text
    }
}
