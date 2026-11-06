//! Where the reader gets its documents.
//!
//! Pattern: Strategy (and Repository-like interface) — the workbench depends on this trait only.
//! It does not know a store, a wiki or a file system; whoever uses the workbench implements it
//! (an Adapter over their data). Paths are opaque names with `/` between the parts.

use fs_components_widgets::Document;

use crate::error::SourceError;

/// One document in a listing.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Where it is, parts separated by `/`.
    pub path: String,
    /// What to call it.
    pub title: String,
}

impl Entry {
    /// An entry.
    pub fn new(path: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            title: title.into(),
        }
    }
}

/// One result of a search.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    /// Where it is.
    pub path: String,
    /// What to call it.
    pub title: String,
}

impl Hit {
    /// A hit.
    pub fn new(path: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            title: title.into(),
        }
    }
}

/// What a link leads to.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkTarget {
    /// A document of this source.
    Document(String),
    /// Something outside (web address, mail); the shell opens it.
    External(String),
    /// Nothing is there; the text says what was meant.
    Missing(String),
}

/// The text of a document as it is edited, and the revision it has.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableText {
    /// The text.
    pub text: String,
    /// Opaque marker of this state of the document; handed back to `save`.
    pub revision: Option<String>,
}

impl EditableText {
    /// An editable text.
    pub fn new(text: impl Into<String>, revision: Option<String>) -> Self {
        Self {
            text: text.into(),
            revision,
        }
    }
}

/// A source of documents. The calls may block briefly (a store lookup); see DESIGN.md for what the
/// reader does about it.
pub trait DocumentSource: Send + Sync {
    /// All documents whose path starts with `prefix` (empty = all), in path order.
    ///
    /// # Errors
    /// [`SourceError`] with a reason a person can read.
    fn entries(&self, prefix: &str) -> Result<Vec<Entry>, SourceError>;
    /// Read one document.
    ///
    /// # Errors
    /// [`SourceError`] with a reason a person can read.
    fn read(&self, path: &str) -> Result<Document, SourceError>;
    /// Follow `href` as written in the document at `from`.
    fn resolve(&self, from: &str, href: &str) -> LinkTarget;
    /// Documents that match `term` by name or content.
    ///
    /// # Errors
    /// [`SourceError`] with a reason a person can read.
    fn find(&self, term: &str) -> Result<Vec<Hit>, SourceError>;
    /// The path to show first, if the source has an opinion.
    fn start(&self) -> Option<String> {
        None
    }

    /// Can the document at `path` be changed? Without it the reader shows no Edit button.
    fn can_edit(&self, _path: &str) -> bool {
        false
    }
    /// The text of the document at `path` as the person edits it.
    ///
    /// # Errors
    /// [`SourceError`]; a source that cannot edit says `Unsupported`.
    fn source_text(&self, _path: &str) -> Result<EditableText, SourceError> {
        Err(SourceError::unsupported("this source cannot be edited"))
    }
    /// Store `text` as the new content of the document at `path`. `revision` is the one given
    /// with the text the person started from; if the document has changed since, answer
    /// [`ErrorKind::Conflict`](crate::ErrorKind::Conflict) and change nothing.
    ///
    /// # Errors
    /// [`SourceError`] with the reason; the reader keeps the person's text.
    fn save(&self, _path: &str, _text: &str, _revision: Option<&str>) -> Result<(), SourceError> {
        Err(SourceError::unsupported("this source cannot be edited"))
    }
    /// Can the document at `path` be exported (written out)? Without it there is no Export button.
    fn can_export(&self, _path: &str) -> bool {
        false
    }
    /// Write the document at `path` out to `target` (chosen by the caller of the reader);
    /// returns a message saying where it went. A target that already holds something is a
    /// [`ErrorKind::Conflict`](crate::ErrorKind::Conflict) unless `overwrite` is set.
    ///
    /// # Errors
    /// [`SourceError`] with the reason.
    fn export(&self, _path: &str, _target: &str, _overwrite: bool) -> Result<String, SourceError> {
        Err(SourceError::unsupported("this source cannot export"))
    }
}
