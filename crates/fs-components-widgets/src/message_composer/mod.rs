//! `message-composer`: write a message, send it as an outcome, lose nothing.
//!
//! Pattern: State + Observer + Adapter + Strategy (`ComposerDrafts`, optional). The text is held
//! exactly as typed; sending reports `Outcome::Send` and clears nothing; the host answers with
//! `sent()` (text cleared if unchanged) or `failed(reason)` (text kept, reason shown). Attachments
//! are a list the host hands in. The composer knows no messenger, no model, no transport.

mod drafts;
mod handle;
mod labels;
mod model;
mod state;
mod view;

pub use handle::{MessageComposer, OnOutcome};
pub use labels::Labels;
pub use model::{Attachment, ComposerDrafts, Outcome, Refusal};
pub use state::Composing;

/// CSS of the composer: only theme variables.
pub const STYLES: &str = r"
.fs-mc { display: flex; flex-direction: column; width: 100%; gap: var(--space-2); padding: var(--space-2); background: var(--surface); border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); }
.fs-mc-area { width: 100%; min-width: 0; }
.fs-mc-notes { display: flex; flex-direction: column; gap: var(--space-1); }
.fs-mc-files { display: flex; flex-direction: row; flex-wrap: wrap; gap: var(--space-2); }
.fs-mc-file { display: flex; flex-direction: row; align-items: center; gap: var(--space-1); padding: 0 var(--space-2); border-radius: var(--radius-sm); border-width: 1px; border-color: var(--border); background: var(--surface-elevated); min-width: 0; }
.fs-mc-file-name { color: var(--text-primary); font-size: var(--text-sm); }
.fs-mc-failed { padding: var(--space-2); border-radius: var(--radius-default); background: var(--error-bg); border-width: 1px; border-color: var(--border-error); }
.fs-mc-note { color: var(--text-primary); font-size: var(--text-sm); }
.fs-mc-buttons { display: flex; width: 100%; flex-direction: row; justify-content: flex-end; gap: var(--space-2); }
";
