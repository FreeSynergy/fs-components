//! The small states of the maps workbench.
//!
//! Pattern: State — plain enums the state machine moves between.

use super::model::PlaceSheet;

/// What the detail area shows.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum Shown {
    /// Nothing is selected.
    Nothing,
    /// This place.
    Place {
        /// The key.
        key: String,
        /// What to show.
        sheet: Box<PlaceSheet>,
    },
    /// This place could not be read.
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

/// The whole workbench, or the compact one (ADR 0037): the map, the places and reading, and picking a value out; no changes.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    /// Everything the source can do.
    #[default]
    Full,
    /// Offers moving, reading and picking only.
    Compact,
}
