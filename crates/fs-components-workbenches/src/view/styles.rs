//! The style sheet of the reader.
//!
//! Pattern: Template — only theme variables; widget sheets are added by the shell.

/// CSS for the reader layout.
pub(crate) fn styles() -> String {
    [LAYOUT, &fs_components_widgets::styles()].concat()
}

const LAYOUT: &str = r"
.fs-rd-root { display: flex; flex-direction: row; background: var(--background); color: var(--text-primary); font-family: var(--font-sans); }
.fs-rd-side { display: flex; flex-direction: column; width: 320px; border-right-width: 1px; border-right-color: var(--border); background: var(--surface); }
.fs-rd-search { padding: var(--space-3); }
.fs-rd-side-body { flex-grow: 1; padding-left: var(--space-2); padding-right: var(--space-2); }
.fs-rd-main { display: flex; flex-direction: column; flex-grow: 1; min-width: 0; }
.fs-rd-bar { display: flex; flex-direction: row; align-items: center; gap: var(--space-3); padding: var(--space-3); border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-rd-path { color: var(--text-secondary); font-size: var(--text-sm); }
.fs-rd-page { width: 100%; padding: var(--space-6); max-width: 900px; }
.fs-rd-notice { padding: var(--space-2) var(--space-3); background: var(--warning-bg); color: var(--text-primary); }
.fs-rd-failed { padding: var(--space-4); background: var(--error-bg); border-radius: var(--radius-default); }
.fs-rd-hit { display: flex; flex-direction: column; padding: var(--space-2); border-radius: var(--radius-default); cursor: pointer; }
.fs-rd-hit:hover { background: var(--surface-elevated); }
.fs-rd-hit-path { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-rd-empty { padding: var(--space-3); color: var(--text-secondary); }
";
