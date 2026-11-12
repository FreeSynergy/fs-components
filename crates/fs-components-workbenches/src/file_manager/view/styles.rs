//! The style sheet of the file manager.
//!
//! Pattern: none — only theme variables; widget sheets are joined here.

/// The style sheet of the workbench and the widgets it uses; add it to the theme of the window that
/// holds a [`FileManagerWidget`](crate::file_manager::FileManagerWidget).
pub fn styles() -> String {
    [LAYOUT, &fs_components_widgets::styles()].concat()
}

const LAYOUT: &str = r"
.fs-files-root { display: flex; background: var(--background); color: var(--text-primary); font-family: var(--font-sans); }
.fs-files-toolbar { padding: var(--space-2) var(--space-3); border-bottom-width: 1px; border-bottom-color: var(--border); background: var(--surface); }
.fs-files-bar-row { align-items: center; gap: var(--space-2); }
.fs-files-search-field { flex-grow: 1; flex-basis: 150px; min-width: 0; }
.fs-files-crumbs { padding: 0 var(--space-3); gap: var(--space-3); border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-files-outside { flex-shrink: 0; color: var(--text-secondary); font-size: var(--text-xs); }
.fs-files-notice { padding: var(--space-2) var(--space-3); background: var(--surface-elevated); color: var(--text-primary); }
.fs-files-problem { padding: var(--space-2) var(--space-3); background: var(--error-bg); color: var(--text-primary); }
.fs-files-tree { padding: var(--space-2); border-right-width: 1px; border-right-color: var(--border); background: var(--surface); }
.fs-files-work { border-right-width: 1px; border-right-color: var(--border); background: var(--surface); }
.fs-files-side-inner { padding: 0; }
.fs-files-item-bar { align-items: center; gap: var(--space-2); padding: var(--space-3); border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-files-preview-room { padding: var(--space-3); min-height: 0; }
.fs-files-band { border-top-width: 1px; border-top-color: var(--border); background: var(--warning-bg); }
.fs-files-band-inner { padding: var(--space-2) var(--space-3); gap: var(--space-1); }
.fs-files-band-row { align-items: center; gap: var(--space-2); }
.fs-files-failed-line { padding: var(--space-1) var(--space-3); background: var(--error-bg); color: var(--text-primary); font-size: var(--text-sm); }
.fs-files-status { padding: 0 var(--space-3); border-top-width: 1px; border-top-color: var(--border); background: var(--surface); color: var(--text-secondary); font-size: var(--text-sm); }
.fs-files-place { padding: var(--space-1) var(--space-3); border-radius: var(--radius-default); cursor: pointer; }
.fs-files-place:hover { background: var(--accent-subtle); }
.fs-files-place-current { background: var(--selection); color: var(--accent); }
.fs-files-tabs-row { background: var(--surface-elevated); }
.fs-files-tabs { padding: 0 var(--space-3); gap: var(--space-1); }
.fs-files-tab { padding: 0 var(--space-3); height: 30px; border-radius: var(--radius-default); background: var(--surface); }
.fs-files-tab-close { padding-left: var(--space-1); }
.fs-files-pathbox { padding: 0 var(--space-2); border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); background: var(--surface); }
.fs-files-new-btn { padding: 0 var(--space-3); border-radius: var(--radius-default); cursor: pointer; }
.fs-files-new-btn:hover { background: var(--accent-subtle); }
.fs-files-place-off { opacity: 0.5; }
.fs-files-icon-btn { border-radius: var(--radius-default); cursor: pointer; }
.fs-files-icon-btn:hover { background: var(--accent-subtle); }
.fs-files-icon-btn-on { background: var(--selection); }
.fs-files-icon-btn-off { opacity: 0.4; cursor: default; }
.fs-files-command-row { padding: 0; gap: 4px; }
.fs-files-empty { padding: var(--space-4); color: var(--text-secondary); }
.fs-files-more { justify-content: center; border-top-width: 1px; border-top-color: var(--border); }
.fs-files-press { padding: var(--space-1) var(--space-3); border-radius: var(--radius-default); background: var(--surface-elevated); cursor: pointer; }
.fs-files-press:hover { background: var(--accent-subtle); }
.fs-files-panel { padding: var(--space-3); gap: var(--space-2); }
.fs-files-section-title { color: var(--text-secondary); font-size: var(--text-sm); font-weight: 600; }
.fs-files-note { color: var(--text-secondary); font-size: var(--text-sm); }
.fs-files-input { padding-bottom: var(--space-1); }
.fs-files-picker-bar { align-items: center; gap: var(--space-2); padding: var(--space-2) var(--space-3); border-top-width: 1px; border-top-color: var(--border); }
";
