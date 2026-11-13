//! The style sheet of the name dialog.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the name dialog: only theme variables.
pub const STYLES: &str = r"
.fs-nd { display: flex; flex-direction: column; width: 100%; max-width: 420px; gap: var(--space-3); padding: var(--space-4); border-radius: var(--radius-lg); background: var(--surface); border-width: 1px; border-color: var(--border); }
.fs-nd-title { color: var(--text-primary); font-size: var(--text-lg); }
.fs-nd-buttons { display: flex; flex-direction: row; justify-content: flex-end; gap: var(--space-2); }
";
