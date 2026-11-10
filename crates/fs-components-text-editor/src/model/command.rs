//! The operations of the editor.
//!
//! Pattern: Command — an enum of operations; `apply` turns one into the [`Edit`] it makes on a
//! given text and selection (one `match`), and the edit knows how to be taken back (`invert`).

use super::edit::Edit;
use super::lines::{heading_plan, prefix_plan, rewrite};
use super::selection::Selection;
use super::wrap::{Wrap, link, toggle};

/// One thing the toolbar can do to the Markdown source.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditCommand {
    /// Bold on or off (`**text**`).
    Bold,
    /// Italic on or off (`*text*`).
    Italic,
    /// Inline code on or off (`` `text` ``).
    Code,
    /// A heading of this level on the lines touched (1 to 6; the same level again takes it off; 0
    /// always takes it off).
    Heading(u8),
    /// A list point (`- `) on the lines touched, or off if they all have one.
    ListItem,
    /// A quotation (`> `) on the lines touched, or off if they all have one.
    Quote,
    /// A link to this address: `[selection](url)`.
    Link(String),
}

impl EditCommand {
    /// The edit this command makes on `text` with `selection`; `None` when it would change nothing.
    pub fn apply(&self, text: &str, selection: Selection) -> Option<Edit> {
        let chars: Vec<char> = text.chars().collect();
        match self {
            Self::Bold => Some(toggle(&chars, selection, Wrap::Bold)),
            Self::Italic => Some(toggle(&chars, selection, Wrap::Italic)),
            Self::Code => Some(toggle(&chars, selection, Wrap::Code)),
            Self::Heading(level) => rewrite(&chars, selection, heading_plan(*level)),
            Self::ListItem => rewrite(&chars, selection, prefix_plan("- ", &["* ", "+ "])),
            Self::Quote => rewrite(&chars, selection, prefix_plan("> ", &[])),
            Self::Link(url) => Some(link(&chars, selection, url)),
        }
    }

    /// The edit that takes back what `applied` (made by `apply`) did.
    pub fn invert(applied: &Edit) -> Edit {
        applied.invert()
    }
}
