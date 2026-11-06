//! What the content area shows, and what following a link led to.
//!
//! Pattern: State — plain enums.

use fs_components_widgets::Document;

/// What the content area shows.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Shown {
    /// Nothing opened yet.
    #[default]
    Nothing,
    /// A document.
    Document {
        /// Its path.
        path: String,
        /// Its content.
        document: Document,
    },
    /// A document that could not be read.
    Failed {
        /// Its path.
        path: String,
        /// Why.
        reason: String,
    },
}

/// What following a link led to; the shell acts on [`Outcome::External`].
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// A document was opened.
    Opened,
    /// An outside address: open it with the system.
    External(String),
    /// Nothing there; a notice is shown.
    Missing,
    /// Not followed: unsaved changes are in the way (a notice says so).
    Blocked,
}
