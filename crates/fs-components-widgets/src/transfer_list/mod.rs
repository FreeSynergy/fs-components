//! `transfer-list`: running uploads and downloads, given as data, with progress, failure and
//! retry reported as outcomes.
//!
//! Pattern: Adapter over `blinc_cn` (progress bar, buttons) plus Observer (the caller's
//! [`Outcome`] callback). The list moves nothing and remembers nothing: the caller hands over the
//! transfers as they are now and answers the outcomes (retry, cancel, dismiss) by changing them.

mod labels;
mod model;
mod view;

pub use labels::Labels;
pub use model::{Action, Direction, Outcome, State, Transfer, TransferList, format_bytes};

/// CSS of the transfer list: only theme variables.
pub const STYLES: &str = r"
.fs-tl { display: flex; flex-direction: column; width: 100%; gap: var(--space-1); }
.fs-tl-row { display: flex; flex-direction: row; align-items: center; gap: var(--space-3); padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); background: var(--surface); border-width: 1px; border-color: var(--border); }
.fs-tl-row-failed { border-color: var(--border-error); background: var(--error-bg); }
.fs-tl-main { display: flex; flex-direction: column; flex-grow: 1; min-width: 0; gap: var(--space-1); overflow: hidden; }
.fs-tl-name { color: var(--text-primary); }
.fs-tl-meta { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-tl-actions { display: flex; flex-direction: row; gap: var(--space-2); flex-shrink: 0; }
.fs-tl-empty, .fs-tl-more { color: var(--text-secondary); padding: var(--space-2); }
";
