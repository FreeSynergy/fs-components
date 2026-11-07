//! `message-list`: a long conversation, virtualised. Only the messages near the view exist.
//!
//! Pattern: Adapter + Strategy (`MessageSource`, `Probe`) + Observer (outcomes) + Memento
//! (scroll position). Neutral data in ([`ChatMessage`]), outcomes out ([`Outcome`]): the list
//! knows no messenger, no model and no transport. Text that grows is the host changing its
//! source and calling `refresh`; the list keeps the place of a reader who is reading older
//! messages and follows only at the bottom. Every message carries its sender's marked kind.

mod estimate;
mod handle;
mod heights;
mod labels;
mod measure;
mod message;
mod model;
mod scroll;
mod styles;
mod view;
mod window;

pub use estimate::estimate;
pub use handle::{MessageList, OnOutcome, Outcome, Probe};
pub use heights::Heights;
pub use labels::Labels;
pub use model::{ChatMessage, Delivery, MessageSource, Reference, SourceError, VecMessages};
pub use styles::STYLES;
pub use window::{OVERSCAN, covered, window};
