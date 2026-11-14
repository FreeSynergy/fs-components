//! The block menu of the Notion-like editor: what "/" offers, how typing filters it, what a choice writes.
//!
//! Pattern: Strategy by block type (pure functions over text; the editor applies the result).
//!
//! First step: block types only (headings 1 to 3, bulleted, numbered and to-do lists, table, quote, code, divider).
//! Content of other programs (embedded through the hub by capability, ADR 0019/0068) is a later step. A choice writes
//! the Markdown of the type into the block, as typing it would.

use fs_components_widgets::SlashLabels;

/// A block type the menu offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    /// A heading of level 1 to 3.
    Heading(u8),
    /// Bulleted list.
    Bullet,
    /// Numbered list.
    Numbered,
    /// List of tasks.
    Task,
    /// Table (header, delimiter row, one row).
    Table,
    /// Quote.
    Quote,
    /// Fenced code.
    Code,
    /// Horizontal rule.
    Divider,
}

/// What a choice does to the block.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Written {
    /// The new text of the block.
    pub text: String,
    /// Byte offset of the cursor in it.
    pub at: usize,
    /// A block of its own follows (a divider holds no text to type into).
    pub block_after: bool,
}

impl Kind {
    /// All types in the order of the menu.
    pub const ALL: [Self; 10] = [
        Self::Heading(1),
        Self::Heading(2),
        Self::Heading(3),
        Self::Bullet,
        Self::Numbered,
        Self::Task,
        Self::Table,
        Self::Quote,
        Self::Code,
        Self::Divider,
    ];

    /// The name in the menu.
    pub fn label(self, words: &SlashLabels) -> &str {
        match self {
            Self::Heading(n) => &words.headings[usize::from(n.clamp(1, 3)) - 1],
            Self::Bullet => &words.bullet,
            Self::Numbered => &words.numbered,
            Self::Task => &words.task,
            Self::Table => &words.table,
            Self::Quote => &words.quote,
            Self::Code => &words.code,
            Self::Divider => &words.divider,
        }
    }

    /// The characters that start the type in Markdown: shown beside the name, and typed after "/" they pick it too.
    pub fn marker(self) -> String {
        match self {
            Self::Heading(n) => "#".repeat(usize::from(n)),
            Self::Bullet => "-".into(),
            Self::Numbered => "1.".into(),
            Self::Task => "[ ]".into(),
            Self::Table => "|".into(),
            Self::Quote => ">".into(),
            Self::Code => "```".into(),
            Self::Divider => "---".into(),
        }
    }

    /// What the block holds after the choice.
    pub fn written(self) -> Written {
        let (text, at) = match self {
            Self::Heading(n) => {
                let t = format!("{} ", "#".repeat(usize::from(n)));
                let at = t.len();
                (t, at)
            }
            Self::Bullet => ("- ".to_owned(), 2),
            Self::Numbered => ("1. ".to_owned(), 3),
            Self::Task => ("- [ ] ".to_owned(), 6),
            // Header, delimiter row, one row; the cursor stands in the first cell of the header.
            Self::Table => ("|  |  |\n| --- | --- |\n|  |  |".to_owned(), 2),
            Self::Quote => ("> ".to_owned(), 2),
            Self::Code => ("```\n\n```".to_owned(), 4),
            Self::Divider => ("---".to_owned(), 3),
        };
        Written {
            text,
            at,
            block_after: self == Self::Divider,
        }
    }
}

/// The filter typed after "/", if the text of a block is a menu request: one line that starts with "/" and holds no space.
pub(super) fn query(text: &str) -> Option<&str> {
    let rest = text.strip_prefix('/')?;
    (!rest.contains(char::is_whitespace)).then_some(rest)
}

/// The types that match `filter`: those whose name contains it (any case) or whose marker starts with it.
pub(super) fn matching(filter: &str, words: &SlashLabels) -> Vec<Kind> {
    let f = filter.to_lowercase();
    Kind::ALL
        .into_iter()
        .filter(|k| {
            f.is_empty() || k.label(words).to_lowercase().contains(&f) || k.marker().starts_with(&f)
        })
        .collect()
}
