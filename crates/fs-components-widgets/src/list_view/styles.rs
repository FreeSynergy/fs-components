//! The style sheet of the list.
//!
//! Pattern: Template — only theme variables, so a theme can change everything.

/// CSS of the list.
pub const STYLES: &str = r"
.fs-list { display: flex; flex-direction: column; min-height: 0; overflow: hidden; background: var(--surface); color: var(--text-primary); }
.fs-list-head { display: flex; flex-direction: row; align-items: center; flex-shrink: 0; border-bottom-width: 1px; border-bottom-color: var(--border); color: var(--text-secondary); font-size: var(--text-sm); }
.fs-list-row { display: flex; flex-direction: row; align-items: center; cursor: pointer; }
.fs-list-row:hover { background: var(--accent-subtle); }
.fs-list-row-selected { background: var(--selection); color: var(--accent); }
.fs-list-cell { display: flex; flex-direction: row; align-items: center; flex-shrink: 0; height: 100%; overflow: hidden; padding-left: var(--space-3); padding-right: var(--space-2); }
.fs-list-failed { display: flex; flex-direction: row; align-items: center; padding-left: var(--space-3); background: var(--error-bg); }
.fs-list-empty { padding: var(--space-3); color: var(--text-secondary); }
";
