//! The style sheet of the calendar workbench.
//!
//! Pattern: Template — only theme variables; widget sheets are joined here.

/// The style sheet of the workbench and the widgets it uses; add it to the theme of the window that
/// holds a [`CalendarWidget`](crate::calendar::CalendarWidget).
pub fn styles() -> String {
    [LAYOUT, &fs_components_widgets::styles()].concat()
}

const LAYOUT: &str = r"
.fs-calendar-root { display: flex; flex-direction: column; background: var(--background); color: var(--text-primary); font-family: var(--font-sans); }
.fs-calendar-top { flex-shrink: 0; border-bottom-width: 1px; border-bottom-color: var(--border); background: var(--surface); }
.fs-calendar-bar { flex-shrink: 0; align-items: center; gap: var(--space-2); padding-left: var(--space-3); padding-right: var(--space-3); min-height: 56px; }
.fs-calendar-range { min-width: 0; font-size: var(--text-lg); font-weight: 600; padding-left: var(--space-2); }
.fs-calendar-detail { flex-shrink: 0; min-width: 0; border-left-width: 1px; border-left-color: var(--border); background: var(--surface); }
.fs-calendar-page { padding: var(--space-4); gap: var(--space-3); }
.fs-calendar-title { align-items: center; gap: var(--space-2); flex-wrap: wrap; }
.fs-calendar-title-text { font-size: var(--text-xl); font-weight: 600; }
.fs-calendar-sheet-mark { padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); background: var(--warning-bg); color: var(--text-primary); font-size: var(--text-sm); }
.fs-calendar-note { padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); background: var(--info-bg); font-size: var(--text-sm); }
.fs-calendar-section { gap: var(--space-2); }
.fs-calendar-section-title { color: var(--text-secondary); font-size: var(--text-sm); font-weight: 600; }
.fs-calendar-field { gap: var(--space-3); align-items: flex-start; }
.fs-calendar-label { width: 96px; flex-shrink: 0; color: var(--text-secondary); font-size: var(--text-sm); }
.fs-calendar-value { min-width: 0; font-size: var(--text-sm); }
.fs-calendar-notice { padding: var(--space-2) var(--space-3); background: var(--warning-bg); color: var(--text-primary); font-size: var(--text-sm); }
.fs-calendar-failed { padding: var(--space-3); background: var(--error-bg); border-radius: var(--radius-default); font-size: var(--text-sm); }
.fs-calendar-pinned { flex-shrink: 0; gap: var(--space-2); padding-left: var(--space-4); padding-right: var(--space-4); padding-top: var(--space-2); }
.fs-calendar-problem { flex-shrink: 0; padding-left: var(--space-3); background: var(--error-bg); border-bottom-width: 1px; border-bottom-color: var(--border); font-size: var(--text-sm); }
.fs-calendar-problem-open { cursor: pointer; }
.fs-calendar-input { padding-bottom: var(--space-2); }
.fs-calendar-flag { padding: var(--space-1) var(--space-3); border-radius: var(--radius-default); cursor: pointer; font-size: var(--text-sm); }
.fs-calendar-flag-on { background: var(--primary); color: var(--text-inverse); }
.fs-calendar-flag-off { background: var(--surface-elevated); border-width: 1px; border-color: var(--border); }
";
