//! `TextId`: the identifier of one translatable text.
//!
//! Pattern: Newtype — a bare `String` could be a sentence or an id; this type is always the id,
//! so a sentence can never be passed where an identifier is asked for.

use std::fmt;

/// Identifier of a text, for example `files.deleted`. Never shown on purpose, except as the
/// visible mark of a missing text (see [`Said::missing`](crate::Said::missing)).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TextId(String);

impl TextId {
    /// Wrap an identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The identifier as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for TextId {
    fn from(id: &str) -> Self {
        Self::new(id)
    }
}

impl fmt::Display for TextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
