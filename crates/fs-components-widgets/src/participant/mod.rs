//! Who wrote it: the marked sender of a message or an agent (ADR 0054).
//!
//! Pattern: Adapter. [`SenderKind`] comes from the data and has no default that
//! looks like a person: unset is `Unknown` and is drawn as unknown. [`chip`] is the one function
//! that draws the mark, so no view can leave it out or style an AI like a person.

mod chip;
mod model;
mod styles;

pub(crate) use chip::chip;
pub use model::{KindLabels, Sender, SenderKind};
pub use styles::STYLES;
