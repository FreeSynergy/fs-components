//! The state of the reader.
//!
//! Pattern: State — [`Shown`] says what the content area shows; [`Reader`] holds the transitions
//! (open, follow, back, search). No drawing here, so it can be tested without a window.

mod model;

pub use model::{Outcome, Reader, Shown};
