//! The style sheet of sortable list.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the sortable list: only theme variables.
pub const STYLES: &str = r"
.fs-sl { display: flex; flex-direction: row; gap: var(--space-3); align-items: flex-start; width: 100%; }
.fs-sl-lane { display: flex; flex-direction: column; flex-grow: 1; flex-basis: 0; min-width: 0; gap: var(--space-1); padding: var(--space-2); border-radius: var(--radius-default); background: var(--surface); border-width: 1px; border-color: var(--border); }
.fs-sl-title { color: var(--text-secondary); font-size: var(--text-sm); padding: var(--space-1); }
.fs-sl-row { display: flex; flex-direction: row; align-items: center; min-width: 0; overflow: hidden; padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); background: var(--surface-elevated); border-width: 2px; border-color: var(--border); color: var(--text-primary); }
.fs-sl-row-cursor { border-color: var(--border-focus); }
.fs-sl-row-grabbed { border-color: var(--primary); background: var(--accent-subtle); }
.fs-sl-row-locked { color: var(--text-tertiary); }
.fs-sl-note { color: var(--text-secondary); font-size: var(--text-xs); padding: var(--space-1) var(--space-2); }
";
