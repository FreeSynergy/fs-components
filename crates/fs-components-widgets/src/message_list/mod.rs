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
mod view;
mod window;

pub use estimate::estimate;
pub use handle::{MessageList, OnOutcome, Outcome, Probe};
pub use heights::Heights;
pub use labels::Labels;
pub use model::{ChatMessage, Delivery, MessageSource, Reference, SourceError, VecMessages};
pub use window::{OVERSCAN, covered, window};

/// CSS of the message list: only theme variables.
pub const STYLES: &str = r"
.fs-ml-col { display: flex; flex-direction: column; width: 100%; }
.fs-ml-msg { display: flex; flex-direction: column; flex-shrink: 0; width: 100%; gap: var(--space-2); padding: var(--space-3); border-left-width: 4px; border-color: var(--border); min-width: 0; }
.fs-ml-k-person { border-color: var(--border); }
.fs-ml-k-software { border-color: var(--secondary); }
.fs-ml-k-ai { border-color: var(--info); background: var(--info-bg); }
.fs-ml-k-unknown { border-color: var(--warning); background: var(--warning-bg); }
.fs-ml-mine { background: var(--accent-subtle); }
.fs-ml-head { display: flex; flex-direction: row; flex-wrap: wrap; align-items: center; gap: var(--space-2); min-width: 0; }
.fs-ml-name { color: var(--text-primary); font-size: var(--text-sm); }
.fs-ml-meta { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-ml-body { display: flex; flex-direction: column; min-width: 0; }
.fs-ml-note { color: var(--text-secondary); font-size: var(--text-sm); }
.fs-ml-delivery-failed { padding: 0 var(--space-2); border-radius: var(--radius-sm); background: var(--error-bg); border-width: 1px; border-color: var(--border-error); }
.fs-ml-tools { display: flex; flex-direction: column; gap: var(--space-2); }
.fs-ml-refs, .fs-ml-reasoning { display: flex; flex-direction: column; align-items: flex-start; gap: var(--space-1); }
.fs-ml-refs { flex-direction: row; flex-wrap: wrap; }
.fs-ml-failure { display: flex; flex-direction: row; flex-wrap: wrap; align-items: center; gap: var(--space-2); padding: var(--space-2); border-radius: var(--radius-default); background: var(--error-bg); border-width: 1px; border-color: var(--border-error); }
.fs-ml-failure-text { color: var(--text-primary); font-size: var(--text-sm); min-width: 0; }
.fs-ml-empty { padding: var(--space-4); color: var(--text-secondary); }
.fs-ml-notice { display: flex; flex-direction: row; justify-content: center; width: 100%; }
";
