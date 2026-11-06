//! The style sheet of the tasks workbench.
//!
//! Pattern: Template — only theme variables; widget sheets are joined here.

/// The style sheet of the workbench and the widgets it uses; add it to the theme of the window that
/// holds a [`TasksWidget`](crate::tasks::TasksWidget).
pub fn styles() -> String {
    [LAYOUT, &fs_components_widgets::styles()].concat()
}

const LAYOUT: &str = r"
.fs-tasks-root { display: flex; background: var(--background); color: var(--text-primary); font-family: var(--font-sans); }
.fs-tasks-toolbar { flex-shrink: 0; padding: var(--space-2) var(--space-3); border-bottom-width: 1px; border-bottom-color: var(--border); background: var(--surface); }
.fs-tasks-bar-row { align-items: center; gap: var(--space-2); }
.fs-tasks-search-field { flex-grow: 1; flex-basis: 140px; min-width: 0; }
.fs-tasks-body { flex-grow: 1; min-height: 0; }
.fs-tasks-work { flex-shrink: 0; border-right-width: 1px; border-right-color: var(--border); background: var(--surface); }
.fs-tasks-main { min-width: 0; }
.fs-tasks-board-bar { flex-shrink: 0; align-items: center; gap: var(--space-3); padding: var(--space-2) var(--space-3); }
.fs-tasks-warning { padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); background: var(--warning-bg); color: var(--text-primary); font-size: var(--text-sm); }
.fs-tasks-more { flex-shrink: 0; display: flex; flex-direction: row; align-items: center; justify-content: center; border-top-width: 1px; border-top-color: var(--border); }
.fs-tasks-more-button { padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); background: var(--surface-elevated); cursor: pointer; }
.fs-tasks-more-button:hover { background: var(--accent-subtle); }
.fs-tasks-empty { padding: var(--space-4); color: var(--text-secondary); }
.fs-tasks-bar { flex-shrink: 0; align-items: center; gap: var(--space-3); padding: var(--space-3); min-height: 60px; border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-tasks-really { flex-shrink: 0; padding: var(--space-3); }
.fs-tasks-page { padding: var(--space-6); gap: var(--space-4); }
.fs-tasks-head { align-items: center; gap: var(--space-3); }
.fs-tasks-title-text { font-size: var(--text-2xl); font-weight: 600; }
.fs-tasks-mark { padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); background: var(--warning-bg); color: var(--text-primary); font-size: var(--text-sm); }
.fs-tasks-note { padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); background: var(--surface-elevated); color: var(--text-secondary); font-size: var(--text-sm); }
.fs-tasks-section { gap: var(--space-2); }
.fs-tasks-section-title { color: var(--text-secondary); font-size: var(--text-sm); font-weight: 600; }
.fs-tasks-choices { gap: var(--space-2); }
.fs-tasks-field { gap: var(--space-3); }
.fs-tasks-label { width: 120px; flex-shrink: 0; color: var(--text-secondary); }
.fs-tasks-value { min-width: 0; }
.fs-tasks-notice { padding: var(--space-2) var(--space-3); background: var(--warning-bg); color: var(--text-primary); }
.fs-tasks-failed { padding: var(--space-4); background: var(--error-bg); border-radius: var(--radius-default); }
.fs-tasks-input { padding-bottom: var(--space-2); }
";
