//! One picture as the grid shows it.
//!
//! Pattern: Value object — plain data the caller fills.

/// A picture.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Picture {
    /// Handed back when the tile is chosen.
    pub key: String,
    /// The name.
    pub name: String,
    /// The media type (`image/png`).
    pub media_type: String,
    /// When it was taken, as an RFC 3339 date or date-time text; empty when not known.
    pub taken: String,
    /// Marks that must be seen at once (for example a sensitive marking), in the caller's words.
    pub marks: Vec<String>,
}

impl Picture {
    /// A picture.
    pub fn new(
        key: impl Into<String>,
        name: impl Into<String>,
        media_type: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            media_type: media_type.into(),
            ..Self::default()
        }
    }

    /// With the time it was taken.
    #[must_use]
    pub fn taken(mut self, taken: impl Into<String>) -> Self {
        self.taken = taken.into();
        self
    }

    /// With these marks.
    #[must_use]
    pub fn with_marks(mut self, marks: Vec<String>) -> Self {
        self.marks = marks;
        self
    }

    /// The short word where a picture will go: `png` for `image/png`.
    pub fn badge(&self) -> String {
        self.media_type
            .rsplit('/')
            .next()
            .unwrap_or_default()
            .split('+')
            .next()
            .unwrap_or_default()
            .to_uppercase()
    }
}
