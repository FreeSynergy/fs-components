//! The style sheet of message list.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the message list: only theme variables.
pub const STYLES: &str = r"
.fs-ml-col { display: flex; flex-direction: column; width: 100%; }
.fs-ml-msg { display: flex; flex-direction: column; flex-shrink: 0; width: 100%; gap: var(--space-2); padding: var(--space-3); min-width: 0; }
.fs-ml-edge { background: var(--border); }
.fs-ml-edge.fs-ml-k-software { background: var(--secondary); }
.fs-ml-edge.fs-ml-k-ai { background: var(--info); }
.fs-ml-edge.fs-ml-k-unknown { background: var(--warning); }
.fs-ml-edge.fs-ml-k-person { background: var(--text-secondary); }
.fs-ml-msg.fs-ml-k-ai { background: var(--info-bg); }
.fs-ml-msg.fs-ml-k-unknown { background: var(--warning-bg); }
.fs-ml-mine { background: var(--surface-elevated); }
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
