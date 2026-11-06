//! `llm-chat`: the chat workbench, with a compact variant (ADR 0037, 0053, 0054).
//!
//! Pattern: Facade + Strategy (`ChatSource`) over the widgets `message-list`, `message-composer`,
//! `agent-tree` and `tool-call`. Open like the reader: it knows no messenger, no model and no
//! store; content comes at run time through the trait, the host feeds it (no network code, no
//! streaming transport here). Every sender is marked, in the compact form too. Every click that
//! matters (a send, a question's answer, the stop) is an [`Outcome`] for the host; nothing is
//! decided or executed here, and the stop itself is the engine's job.

mod handle;
mod labels;
mod outcome;
mod source;
mod styles;
mod view;

pub use handle::{LlmChat, OnOutcome};
pub use labels::Labels;
pub use outcome::Outcome;
pub use source::{ChatSource, Pending, StopControl, StopState};
pub use styles::styles;
