//! `Said`: the answer of [`Words::say`](crate::Words::say).
//!
//! Pattern: Value object — found or missing is a type, so callers (and logs) can tell them apart
//! without comparing text. The fields are private and only the constructors make a value, so a
//! `Said` can never carry empty or blank text: such an answer becomes a missing mark.

use std::fmt;

use crate::TextId;

/// A sentence, or the visible mark that it could not be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Said {
    found: bool,
    text: String,
}

impl Said {
    /// A finished sentence for `id`. Empty or blank text is not a sentence: it becomes
    /// [`Said::missing`], so a reader never sees nothing.
    pub fn found(id: &TextId, text: impl Into<String>) -> Self {
        let text = text.into();
        if text.trim().is_empty() {
            Self::missing(id)
        } else {
            Self { found: true, text }
        }
    }

    /// The mark of a text that could not be made: it shows the identifier (a blank identifier is
    /// shown as `(no id)`), so a reader and a translator can see which text it was.
    pub fn missing(id: &TextId) -> Self {
        let text = if id.as_str().trim().is_empty() {
            "(no id)".to_owned()
        } else {
            id.as_str().to_owned()
        };
        Self { found: false, text }
    }

    /// What to show the reader: the sentence, or the identifier. Never blank.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// True if this is a made sentence, false for the missing mark.
    pub fn is_found(&self) -> bool {
        self.found
    }
}

impl fmt::Display for Said {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}
