//! Where the reader gets its content.
//!
//! Pattern: Strategy (and Repository-like read interface) — the workbench depends on this trait
//! only. It does not know a store, a wiki or a file system; whoever uses the workbench implements
//! it (an Adapter over their data). Paths are opaque names with `/` between the parts.

use fs_components_widgets::Document;

/// One document in a listing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Where it is, parts separated by `/`.
    pub path: String,
    /// What to call it.
    pub title: String,
}

/// One result of a search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    /// Where it is.
    pub path: String,
    /// What to call it.
    pub title: String,
}

/// What a link leads to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkTarget {
    /// A document of this source.
    Page(String),
    /// Something outside (web address, mail); the shell opens it.
    External(String),
    /// Nothing is there; the text says what was meant.
    Missing(String),
}

/// A document could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadError(pub String);

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A source of documents. The calls may block briefly (a store lookup); the reader calls them
/// from the interface thread.
pub trait DataSource: Send + Sync {
    /// All documents whose path starts with `prefix` (empty = all), in path order.
    ///
    /// # Errors
    /// [`ReadError`] with a reason a person can read.
    fn entries(&self, prefix: &str) -> Result<Vec<Entry>, ReadError>;
    /// Read one document.
    ///
    /// # Errors
    /// [`ReadError`] with a reason a person can read.
    fn read(&self, path: &str) -> Result<Document, ReadError>;
    /// Follow `href` as written in the document at `from`.
    fn resolve(&self, from: &str, href: &str) -> LinkTarget;
    /// Documents that match `term` by name or content.
    ///
    /// # Errors
    /// [`ReadError`] with a reason a person can read.
    fn find(&self, term: &str) -> Result<Vec<Hit>, ReadError>;
    /// The path to show first, if the source has an opinion.
    fn start(&self) -> Option<String> {
        None
    }
}
