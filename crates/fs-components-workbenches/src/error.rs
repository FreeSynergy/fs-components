//! The error of a document source (it lives in the widgets crate, shared with the draft store).
//!
//! Pattern: none — a re-export; the old path keeps working for every program that names it.

pub use fs_components_widgets::source_error::{ErrorKind, SourceError};
