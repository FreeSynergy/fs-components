//! The style sheet of file preview.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the preview: only theme variables.
pub const STYLES: &str = r"
.fs-preview { min-height: 0; gap: var(--space-2); color: var(--text-primary); }
.fs-preview-title { font-size: var(--text-lg); font-weight: 600; }
.fs-preview-marks { gap: var(--space-1); }
.fs-preview-mark { padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); background: var(--warning-bg); color: var(--text-primary); font-size: var(--text-sm); }
.fs-preview-fact { gap: var(--space-2); font-size: var(--text-sm); }
.fs-preview-label { width: 96px; flex-shrink: 0; color: var(--text-secondary); }
.fs-preview-value { min-width: 0; }
.fs-preview-text { padding: var(--space-2); background: var(--surface-elevated); border-radius: var(--radius-default); font-family: var(--font-mono); font-size: var(--text-xs); }
.fs-preview-note { padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); background: var(--surface-elevated); color: var(--text-secondary); font-size: var(--text-sm); }
.fs-preview-refused { padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); background: var(--error-bg); font-size: var(--text-sm); }
";
