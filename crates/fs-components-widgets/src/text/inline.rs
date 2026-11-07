//! Inline text with marks.
//!
//! Pattern: none — plain data. Marks are stand-off ranges over the text (like the text
//! layers of the wiki), counted in characters, so the model has no opinion about encodings.

/// What a mark does to the characters it covers.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarkKind {
    /// Emphasis (usually italic).
    Emphasis,
    /// Strong emphasis (usually bold).
    Strong,
    /// Strike-through.
    Strike,
    /// A code span (monospace).
    Code,
    /// A link; `href` is exactly as written. What it means is decided by whoever follows it.
    Link {
        /// The target as written.
        href: String,
    },
}

/// A mark over a range of characters (`start..end`, end exclusive).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mark {
    /// What it does.
    pub kind: MarkKind,
    /// First character.
    pub start: usize,
    /// One past the last character.
    pub end: usize,
}

impl Mark {
    /// A mark of `kind` over `start..end`.
    pub fn new(kind: MarkKind, start: usize, end: usize) -> Self {
        Self { kind, start, end }
    }
}

/// A run of text with marks. Marks may overlap and nest.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inline {
    /// The characters.
    pub text: String,
    /// The marks over them.
    pub marks: Vec<Mark>,
}

impl Inline {
    /// Text without marks.
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            marks: Vec::new(),
        }
    }

    /// Text with marks.
    pub fn new(text: impl Into<String>, marks: Vec<Mark>) -> Self {
        Self {
            text: text.into(),
            marks,
        }
    }

    /// Is there no text?
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}
