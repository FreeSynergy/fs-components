//! A selection in characters.
//!
//! Pattern: Value object — `start <= end` always holds, counted in characters (not bytes), so a
//! multi-byte character is one step.

/// A range of the text in characters; `start == end` is a caret.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    /// First selected character.
    pub start: usize,
    /// One past the last selected character.
    pub end: usize,
}

impl Selection {
    /// The range between `a` and `b` (either order).
    pub fn new(a: usize, b: usize) -> Self {
        Self {
            start: a.min(b),
            end: a.max(b),
        }
    }

    /// A caret at `at`.
    pub fn caret(at: usize) -> Self {
        Self { start: at, end: at }
    }

    /// Nothing selected?
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// The selection cut to a text of `len` characters.
    #[must_use]
    pub fn clamp(self, len: usize) -> Self {
        Self {
            start: self.start.min(len),
            end: self.end.min(len),
        }
    }
}
