//! A draft as backed up, and where backups go.
//!
//! Pattern: Strategy (`DraftStore`, supplied by the host) and a value object (`Draft`).

use crate::source_error::SourceError;

/// A draft as backed up.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Draft {
    /// The key of the backup: the document's path, or (for the second slot, text typed while an
    /// earlier offer waited) the path with a suffix. A store treats it as an opaque key and gives
    /// it back unchanged; it does not interpret it as a document path.
    pub path: String,
    /// The text of the draft.
    pub text: String,
    /// The text the edit began from.
    pub original: String,
    /// The revision that text came from.
    pub revision: Option<String>,
}

impl Draft {
    /// A draft of the document at `path`.
    pub fn new(
        path: impl Into<String>,
        text: impl Into<String>,
        original: impl Into<String>,
        revision: Option<String>,
    ) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
            original: original.into(),
            revision,
        }
    }
}

/// Where drafts are backed up. Implementations write atomically (a crash in the middle must leave
/// the old backup or the new one, never half of one).
pub trait DraftStore: Send + Sync {
    /// Keep `draft` as the backup of its document, replacing an older one.
    ///
    /// # Errors
    /// [`SourceError`] with a reason a person can read.
    fn save(&self, draft: &Draft) -> Result<(), SourceError>;
    /// The backup of the document at `path`, if there is one.
    ///
    /// # Errors
    /// [`SourceError`] with a reason a person can read.
    fn load(&self, path: &str) -> Result<Option<Draft>, SourceError>;
    /// Remove the backup of the document at `path` (no error if there is none).
    ///
    /// # Errors
    /// [`SourceError`] with a reason a person can read.
    fn clear(&self, path: &str) -> Result<(), SourceError>;
}
