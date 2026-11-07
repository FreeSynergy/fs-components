//! The style sheet of the text view.
//!
//! Pattern: none — only theme variables (`var(--…)`), no literal colours or font names.

/// CSS for the text view; added to the theme by the shell.
pub const STYLES: &str = r"
.fs-tv { display: flex; flex-direction: column; width: 100%; }
.fs-tv-block { width: 100%; margin-bottom: var(--space-3); }
.fs-tv-heading { padding-top: var(--space-3); }
.fs-tv-run { max-width: 100%; min-width: 0; }
.fs-tv-codespan { background: var(--surface-elevated); border-radius: var(--radius-sm); padding-left: 2px; padding-right: 2px; }
.fs-tv-quote { border-left-width: 3px; border-left-color: var(--border); padding-left: var(--space-3); }
.fs-tv-code { background: var(--surface-elevated); border-radius: var(--radius-default); padding: var(--space-3); display: flex; flex-direction: column; }
.fs-tv-rule { height: 1px; background: var(--border); margin-top: var(--space-2); }
.fs-tv-item { display: flex; flex-direction: row; align-items: flex-start; gap: var(--space-1); width: 100%; }
.fs-tv-item .fs-tv-block { margin-bottom: var(--space-1); }
.fs-tv-marker { min-width: 16px; color: var(--text-secondary); }
.fs-tv-table { border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); }
.fs-tv-row { display: flex; flex-direction: row; align-items: stretch; width: 100%; border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-tv-head { background: var(--surface-elevated); }
.fs-tv-cell { display: flex; flex-direction: column; justify-content: flex-start; padding: var(--space-2); min-width: 0; }
";
