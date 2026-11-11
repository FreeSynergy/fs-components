//! `tool-call`: what an agent is doing, one card per call, updated in place by id.
//!
//! Pattern: Adapter + Observer + State (the calls are kept in `ToolCalls`). The kind of the call (reads, writes,
//! runs a command, connected tool) is the first thing on the card, before the name and before any
//! detail. A question shows its options; a click is reported as an [`Outcome`] and decides
//! nothing. Stopping an agent is not here: that is the engine's job, the host shows its control.

mod buttons;
mod card;
mod clock;
mod entry;
#[cfg(test)]
mod entry_tests;
mod guard;
mod handle;
mod labels;
mod model;
mod store;
mod styles;

pub(crate) use card::{Ctx, Sinks, card};
pub use clock::{ARM_DELAY_MS, Clock, MIN_DELAY_MS, SystemClock};
pub use guard::Approvals;
pub(crate) use guard::refresh_later;
pub use styles::STYLES;

pub use handle::{OnOutcome, Outcome, ToolCallView, Verify};
pub use labels::Labels;
pub use model::{
    Decision, ToolCall, ToolChoice, ToolEnd, ToolKind, ToolOption, ToolRisk, ToolState,
};
pub use store::{Applied, ToolCalls};
