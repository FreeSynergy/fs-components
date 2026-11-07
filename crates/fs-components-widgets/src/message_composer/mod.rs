//! `message-composer`: write a message, send it as an outcome, lose nothing.
//!
//! Pattern: State + Observer + Adapter + Strategy (the shared `DraftStore`, optional). The text is held
//! exactly as typed; sending reports `Outcome::Send` and clears nothing; the host answers with
//! `sent()` (text cleared if unchanged) or `failed(reason)` (text kept, reason shown). Attachments
//! are a list the host hands in. The composer knows no messenger, no model, no transport.

mod drafts;
mod handle;
mod labels;
mod model;
mod state;
mod styles;
mod view;

pub use handle::{MessageComposer, OnOutcome};
pub use labels::Labels;
pub use model::{Attachment, Outcome, Refusal};
pub use state::Composing;
pub use styles::STYLES;
