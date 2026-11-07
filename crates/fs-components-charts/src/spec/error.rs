//! Why a spec was refused.
//!
//! Pattern: none — an error carries the JSON path of the offending place and a message
//! that names the feature, so a person can fix the spec (or know it is outside the subset).

use std::fmt;

/// What kind of refusal this is.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// The text is not JSON.
    NotJson,
    /// Valid Vega-Lite (or unknown to us), but outside the accepted subset: not drawn at all.
    OutsideSubset,
    /// Inside the subset's vocabulary but wrong: a wrong type, a missing part, a contradiction.
    Invalid,
}

/// A refused spec: where, what kind, and a message naming the feature.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecError {
    /// The kind of refusal.
    pub kind: ErrorKind,
    /// JSON pointer to the place (`/encoding/x/bin`); empty for the whole document.
    pub path: String,
    /// What is wrong, in a sentence.
    pub message: String,
}

impl SpecError {
    pub(crate) fn outside(path: &str, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::OutsideSubset, path, message)
    }

    pub(crate) fn invalid(path: &str, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Invalid, path, message)
    }

    pub(crate) fn new(kind: ErrorKind, path: &str, message: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.to_string(),
            message: message.into(),
        }
    }
}

impl fmt::Display for SpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let place = if self.path.is_empty() {
            "spec"
        } else {
            &self.path
        };
        let what = match self.kind {
            ErrorKind::NotJson => "not JSON",
            ErrorKind::OutsideSubset => "outside the supported subset",
            ErrorKind::Invalid => "invalid",
        };
        write!(f, "{place}: {what}: {}", self.message)
    }
}

impl std::error::Error for SpecError {}

/// Result of reading a spec.
pub type Result<T> = std::result::Result<T, SpecError>;
