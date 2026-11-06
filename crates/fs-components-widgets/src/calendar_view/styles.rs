//! The style sheet of the calendar view.
//!
//! Pattern: Template — only theme variables, so a theme can change everything. The six category
//! colours are the theme's status colours; which item has which is the caller's matter.

/// CSS of the calendar view.
pub const STYLES: &str = r"
.fs-calendar { background: var(--background); color: var(--text-primary); overflow: hidden; }
.fs-calendar-head { flex-shrink: 0; border-bottom-width: 1px; border-bottom-color: var(--border); color: var(--text-secondary); font-size: var(--text-sm); }
.fs-calendar-head-cell { flex-shrink: 0; gap: var(--space-1); padding-left: var(--space-2); border-left-width: 1px; border-left-color: var(--border); }
.fs-calendar-day { cursor: pointer; border-right-width: 1px; border-right-color: var(--border); border-bottom-width: 1px; border-bottom-color: var(--border); padding-left: var(--space-1); padding-top: 2px; }
.fs-calendar-day:hover { background: var(--accent-subtle); }
.fs-calendar-outside { color: var(--text-tertiary); }
.fs-calendar-number { height: 20px; flex-shrink: 0; border-radius: var(--radius-default); font-size: var(--text-sm); }
.fs-calendar-selected { background: var(--selection); }
.fs-calendar-today .fs-calendar-number { background: var(--primary); color: var(--text-inverse); }
.fs-calendar-item { padding-left: 4px; padding-right: 4px; gap: 4px; border-radius: var(--radius-sm); font-size: var(--text-xs); color: var(--text-primary); cursor: pointer; align-items: flex-start; border-left-width: 3px; }
.fs-calendar-item:hover { opacity: 0.85; }
.fs-calendar-item-selected { border-width: 2px; border-color: var(--primary); }
.fs-calendar-item-c0 { background: var(--success-bg); border-left-color: var(--success); }
.fs-calendar-item-c1 { background: var(--info-bg); border-left-color: var(--info); }
.fs-calendar-item-c2 { background: var(--surface-elevated); border-left-color: var(--border-hover); color: var(--text-secondary); }
.fs-calendar-item-c3 { background: var(--warning-bg); border-left-color: var(--warning); }
.fs-calendar-item-c4 { background: var(--error-bg); border-left-color: var(--error); }
.fs-calendar-item-c5 { background: var(--accent-subtle); border-left-color: var(--accent); }
.fs-calendar-mark { padding-left: 3px; padding-right: 3px; border-radius: var(--radius-sm); background: var(--warning); color: var(--text-primary); font-size: var(--text-xs); }
.fs-calendar-more { padding-left: var(--space-1); border-radius: var(--radius-sm); background: var(--surface-elevated); color: var(--text-secondary); font-size: var(--text-xs); cursor: pointer; }
.fs-calendar-more:hover { background: var(--accent-subtle); }
.fs-calendar-all-day { flex-shrink: 0; border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-calendar-gutter { flex-shrink: 0; color: var(--text-tertiary); font-size: var(--text-xs); padding-right: var(--space-1); padding-top: 2px; }
.fs-calendar-column { cursor: pointer; border-left-width: 1px; border-left-color: var(--border); }
.fs-calendar-hour { border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-calendar-now { background: var(--error); }
.fs-calendar-agenda-day { color: var(--text-secondary); font-size: var(--text-sm); font-weight: 600; cursor: pointer; border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-calendar-agenda-today { color: var(--primary); }
.fs-calendar-agenda-row { gap: var(--space-2); padding-top: 2px; }
.fs-calendar-agenda-time { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-calendar-empty { padding: var(--space-6); color: var(--text-secondary); }
";
