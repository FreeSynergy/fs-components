//! `tool-call`: what an agent is doing, one card per call, updated in place by id.
//!
//! Pattern: Adapter + Observer + Repository (in memory). The kind of the call (reads, writes,
//! runs a command, connected tool) is the first thing on the card, before the name and before any
//! detail. A question shows its options; a click is reported as an [`Outcome`] and decides
//! nothing. Stopping an agent is not here: that is the engine's job, the host shows its control.

mod buttons;
mod card;
mod clock;
mod entry;
mod guard;
mod handle;
mod labels;
mod model;
mod store;

pub(crate) use card::{Ctx, Sinks, card};
pub use clock::{ARM_DELAY_MS, Clock, MIN_DELAY_MS, SystemClock};
pub use guard::Approvals;
pub(crate) use guard::refresh_later;

pub use handle::{OnOutcome, Outcome, ToolCallView, Verify};
pub use labels::Labels;
pub use model::{
    Decision, ToolCall, ToolChoice, ToolEnd, ToolKind, ToolOption, ToolRisk, ToolState,
};
pub use store::{Applied, ToolCalls};

/// CSS of the tool call: only theme variables.
pub const STYLES: &str = r"
.fs-tcl { display: flex; flex-direction: column; width: 100%; gap: var(--space-2); }
.fs-tc { display: flex; flex-direction: column; gap: var(--space-2); min-width: 0; padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); background: var(--surface); border-width: 1px; border-color: var(--border); }
.fs-tc-question { border-color: var(--warning); border-width: 2px; background: var(--warning-bg); }
.fs-tc-failed { border-color: var(--border-error); background: var(--error-bg); }
.fs-tc-head { display: flex; flex-direction: row; flex-wrap: wrap; align-items: center; gap: var(--space-2); min-width: 0; }
.fs-tc-name { color: var(--text-primary); font-size: var(--text-sm); min-width: 0; }
.fs-tc-kind, .fs-tc-state, .fs-tc-risk { display: flex; flex-direction: row; flex-shrink: 0; padding: 0 var(--space-2); border-radius: var(--radius-sm); border-width: 1px; border-color: var(--border); }
.fs-tc-chip-text { font-size: var(--text-xs); color: var(--text-primary); }
.fs-tc-kind-reads { background: var(--info-bg); border-color: var(--info); }
.fs-tc-kind-writes { background: var(--warning-bg); border-color: var(--warning); }
.fs-tc-kind-runs { background: var(--error-bg); border-color: var(--error); }
.fs-tc-kind-connected { background: var(--surface-elevated); border-color: var(--secondary); }
.fs-tc-kind-unknown { background: var(--surface); border-color: var(--text-tertiary); }
.fs-tc-risk-ok { border-color: var(--success); }
.fs-tc-risk-changes { border-color: var(--warning); }
.fs-tc-risk-high { border-color: var(--error); background: var(--error-bg); }
.fs-tc-risk-unknown { border-color: var(--text-tertiary); }
.fs-tc-summary { color: var(--text-primary); font-size: var(--text-sm); }
.fs-tc-options { display: flex; flex-direction: row; flex-wrap: wrap; gap: var(--space-2); }
.fs-tc-details { display: flex; flex-direction: column; gap: var(--space-1); min-width: 0; padding: var(--space-2); border-radius: var(--radius-sm); background: var(--surface-elevated); }
.fs-tc-caption { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-tc-btn { display: flex; flex-direction: row; align-items: center; padding: var(--space-1) var(--space-3); border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); min-height: 28px; }
.fs-tc-btn-yes { background: var(--primary); border-color: var(--primary); }
.fs-tc-btn-no { background: var(--surface); }
.fs-tc-btn-cursor { border-color: var(--border-focus); border-width: 2px; }
.fs-tc-btn-off { background: var(--surface-elevated); border-color: var(--border); opacity: 0.45; }
.fs-tc-btn-text { color: var(--text-primary); font-size: var(--text-sm); }
.fs-tc-refused { padding: var(--space-2); border-radius: var(--radius-default); background: var(--error-bg); border-width: 1px; border-color: var(--border-error); }
.fs-tc-code { color: var(--text-primary); font-size: var(--text-sm); }
";
