//! The state of the reader.
//!
//! Pattern: State — [`Shown`] says what the content area shows; [`Reader`] holds the transitions
//! (open, follow, back, search, edit). No drawing here, so it can be tested without a window.

mod backup;
mod draft;
mod edit;
mod exact;
mod model;
mod state;

pub use draft::{Draft, DraftStore};
pub use edit::Editing;
pub use exact::lossy_note;
pub use model::Reader;
pub use state::{Outcome, Shown};
