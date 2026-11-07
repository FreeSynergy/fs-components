//! The style sheet of participant.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the sender mark: only theme variables. The kinds differ by colour, border and word.
pub const STYLES: &str = r"
.fs-pk { display: flex; flex-direction: row; flex-shrink: 0; padding: 0 var(--space-2); border-radius: var(--radius-sm); border-width: 1px; border-color: var(--border); }
.fs-pk-text { font-size: var(--text-xs); color: var(--text-primary); }
.fs-pk-person { background: var(--surface); border-color: var(--border); }
.fs-pk-software { background: var(--surface-elevated); border-color: var(--secondary); }
.fs-pk-ai { background: var(--info-bg); border-color: var(--info); }
.fs-pk-unknown { background: var(--warning-bg); border-color: var(--warning); }
";
