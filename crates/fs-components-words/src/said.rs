//! `Said`: the answer of [`Words::say`](crate::Words::say).
//!
//! Pattern: Value object — found or missing is a type, so callers (and logs) can tell them apart
//! without comparing text. The text itself is never empty.

use std::fmt;

/// A sentence, or the visible mark that it could not be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Said {
    /// The finished sentence.
    Found(String),
    /// The text, or a value it needs, was missing; holds the identifier so a reader (and a
    /// translator) can see which text it was.
    Missing(String),
}

impl Said {
    /// A missing mark for `id`; an empty id is shown as `(no id)`.
    pub fn missing(id: &str) -> Self {
        Self::Missing(if id.is_empty() {
            "(no id)".to_owned()
        } else {
            id.to_owned()
        })
    }

    /// What to show the reader: the sentence, or the identifier. Never empty.
    pub fn text(&self) -> &str {
        match self {
            Self::Found(s) | Self::Missing(s) => s,
        }
    }

    /// True for [`Said::Found`].
    pub fn is_found(&self) -> bool {
        matches!(self, Self::Found(_))
    }
}

impl fmt::Display for Said {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.text())
    }
}
