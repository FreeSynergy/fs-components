//! The small states of the tasks workbench.
//!
//! Pattern: State — plain enums the state machine moves between.

use super::model::TaskSheet;

/// What the detail area shows.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shown {
    /// No task is selected.
    Nothing,
    /// This task.
    Task {
        /// The key.
        key: String,
        /// What to show.
        sheet: TaskSheet,
    },
    /// This task could not be read.
    Failed {
        /// The key.
        key: String,
        /// Why not, in words.
        reason: String,
    },
}

/// What came of asking to go elsewhere.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Done.
    Done,
    /// Refused: there are unsaved changes.
    Blocked,
}

/// The whole workbench, or the compact "my next tasks" (ADR 0037): open tasks, list and detail, no
/// changes.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    /// Everything the source can do.
    #[default]
    Full,
    /// Open tasks only, list and detail, no changes.
    Compact,
}
