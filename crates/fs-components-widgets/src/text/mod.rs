//! The neutral description of a text: what [`crate::TextView`] draws.
//!
//! Pattern: none (declares modules and re-exports only) — declares the model modules and re-exports them.

mod block;
mod inline;
mod plain;

pub use block::{Align, Block, Document, ListItem};
pub use inline::{Inline, Mark, MarkKind};
