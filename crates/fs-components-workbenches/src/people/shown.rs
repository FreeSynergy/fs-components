//! The small states of the people workbench.
//!
//! Pattern: State — plain enums the state machine moves between.

use super::model::PersonSheet;

/// What the detail area shows.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shown {
    /// Nobody is selected.
    Nothing,
    /// This person.
    Person {
        /// The key.
        key: String,
        /// What to show.
        sheet: PersonSheet,
    },
    /// This person could not be read.
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

/// The whole workbench, or the compact one (ADR 0037): list and detail, no changes.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    /// Everything the source can do.
    #[default]
    Full,
    /// List and detail only.
    Compact,
}
