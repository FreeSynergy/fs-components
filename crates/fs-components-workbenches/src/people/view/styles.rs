//! The style sheet of the people workbench.
//!
//! Pattern: Template — only theme variables; widget sheets are joined here.

/// The style sheet of the workbench and the widgets it uses; add it to the theme of the window that
/// holds a [`PeopleWidget`](crate::people::PeopleWidget).
pub fn styles() -> String {
    [LAYOUT, &fs_components_widgets::styles()].concat()
}

const LAYOUT: &str = r"
.fs-people-root { display: flex; background: var(--background); color: var(--text-primary); font-family: var(--font-sans); }
.fs-people-side { flex-shrink: 0; border-right-width: 1px; border-right-color: var(--border); background: var(--surface); }
.fs-people-search { flex-shrink: 0; align-items: center; gap: var(--space-2); padding-left: var(--space-3); padding-right: var(--space-3); }
.fs-people-search-row { gap: var(--space-2); }
.fs-people-search-field { flex-grow: 1; min-width: 0; }
.fs-people-more { flex-shrink: 0; display: flex; flex-direction: row; align-items: center; justify-content: center; border-top-width: 1px; border-top-color: var(--border); }
.fs-people-more-button { padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); background: var(--surface-elevated); cursor: pointer; }
.fs-people-more-button:hover { background: var(--accent-subtle); }
.fs-people-main { min-width: 0; }
.fs-people-bar { flex-shrink: 0; align-items: center; gap: var(--space-3); padding: var(--space-3); min-height: 60px; border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-people-really { flex-shrink: 0; padding: var(--space-3); }
.fs-people-page { padding: var(--space-4); gap: var(--space-4); }
.fs-people-spacer { flex-grow: 1; }
.fs-people-head { align-items: center; gap: var(--space-3); flex-wrap: wrap; }
.fs-people-title-text { font-size: var(--text-2xl); font-weight: 600; }
.fs-people-mark { padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); background: var(--warning-bg); color: var(--text-primary); font-size: var(--text-sm); }
.fs-people-section { gap: var(--space-2); }
.fs-people-section-title { color: var(--text-secondary); font-size: var(--text-sm); font-weight: 600; }
.fs-people-field { gap: var(--space-3); }
.fs-people-label { width: 120px; flex-shrink: 0; color: var(--text-secondary); }
.fs-people-value { min-width: 0; }
.fs-people-notice { padding: var(--space-2) var(--space-3); background: var(--warning-bg); color: var(--text-primary); }
.fs-people-failed { padding: var(--space-4); background: var(--error-bg); border-radius: var(--radius-default); }
.fs-people-input { padding-bottom: var(--space-2); }
";
