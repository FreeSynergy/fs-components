//! The error of a document source.
//!
//! Pattern: plain error value with a kind — callers (the reader, a host program) decide by the
//! kind (for example a conflict keeps the draft), people read the message.

/// What went wrong, roughly.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// There is no such document.
    NotFound,
    /// Someone else changed the document, or the target is taken.
    Conflict,
    /// The source does not offer this (for example it cannot be edited).
    Unsupported,
    /// Anything else (storage, files).
    Failed,
}

/// A document source could not do what was asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceError {
    /// The kind.
    pub kind: ErrorKind,
    /// A reason a person can read.
    pub message: String,
}

impl SourceError {
    /// An error of `kind` with `message`.
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// [`ErrorKind::NotFound`].
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, message)
    }

    /// [`ErrorKind::Conflict`].
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Conflict, message)
    }

    /// [`ErrorKind::Unsupported`].
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Unsupported, message)
    }

    /// [`ErrorKind::Failed`].
    pub fn failed(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Failed, message)
    }
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SourceError {}
