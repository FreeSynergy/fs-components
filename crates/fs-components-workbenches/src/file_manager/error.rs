//! What can go wrong in a file source.
//!
//! Pattern: plain error value with a kind — the workbench decides by the kind (a taken name asks a
//! question, a changed file offers a rescan), people read the message. The three named kinds are the
//! problem types of the contract; they are shown as such, not as "something failed".

/// What went wrong, roughly.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileProblem {
    /// There is no such thing (any more).
    NotFound,
    /// The name is taken (`name-conflict`).
    NameConflict,
    /// The name is not one this backend can hold (`invalid-name`).
    InvalidName,
    /// The file was changed outside the program (`changed-outside`).
    ChangedOutside,
    /// A marked file needs an explicit act (ADR 0029); nothing happened.
    MarkedNeedsRelease,
    /// The source does not offer this.
    Unsupported,
    /// Anything else.
    Failed,
}

/// A file source could not do what was asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileError {
    /// The kind.
    pub problem: FileProblem,
    /// A reason a person can read.
    pub message: String,
}

impl FileError {
    /// An error of this kind.
    pub fn new(problem: FileProblem, message: impl Into<String>) -> Self {
        Self {
            problem,
            message: message.into(),
        }
    }

    /// [`FileProblem::Failed`].
    pub fn failed(message: impl Into<String>) -> Self {
        Self::new(FileProblem::Failed, message)
    }

    /// [`FileProblem::Unsupported`].
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(FileProblem::Unsupported, message)
    }
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for FileError {}
