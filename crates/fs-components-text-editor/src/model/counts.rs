//! Counting characters and words.
//!
//! Pattern: Value object — numbers computed from a text.

/// How much text there is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    /// Characters (not bytes, not grapheme clusters), line breaks included.
    pub chars: usize,
    /// Words: runs of characters between white space.
    pub words: usize,
}

impl Counts {
    /// The counts of `text`.
    pub fn of(text: &str) -> Self {
        Self {
            chars: text.chars().count(),
            words: text.split_whitespace().count(),
        }
    }
}
