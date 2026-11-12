//! The look of the plan widgets: theme variables for colour, its own sizes for text (larger than the compact widgets).
//!
//! Pattern: none — a style sheet. The sizes are literal pixels of this sheet only; no older widget reads them.

/// CSS of the plan widgets.
pub const STYLES: &str = r"
.fs-plan-root { background: var(--background); color: var(--text-primary); font-family: var(--font-sans); }
.fs-plan-message { padding: 24px; font-size: 16px; color: var(--text-secondary); }
.fs-plan-main { min-width: 0; }
.fs-plan-muted { opacity: 0.55; }
.fs-plan-note { color: var(--text-secondary); font-size: 13px; }
.fs-plan-code { color: var(--text-secondary); font-size: 13px; margin-right: 8px; }
.fs-plan-chips { display: flex; flex-direction: row; flex-wrap: wrap; gap: 6px; }
.fs-plan-chip { padding: 2px 8px; border-radius: 6px; }
.fs-plan-chip-text { font-size: 12px; font-weight: 600; }
.fs-plan-chip-neutral { background: var(--accent-subtle); color: var(--text-secondary); }
.fs-plan-chip-accent { background: var(--accent-subtle); color: var(--accent); }
.fs-plan-chip-warning { background: var(--warning-bg); color: var(--text-primary); }
.fs-plan-chip-danger { background: var(--error-bg); color: var(--error); }
.fs-plan-chip-success { background: var(--success-bg); color: var(--success); }
.fs-plan-tabs { display: flex; flex-direction: row; gap: 4px; padding-left: 16px; padding-top: 8px; border-bottom-width: 1px; border-bottom-color: var(--border); flex-shrink: 0; }
.fs-plan-tab { padding: 10px 14px; cursor: pointer; border-bottom-width: 2px; border-bottom-color: transparent; }
.fs-plan-tab-text { font-size: 15px; color: var(--text-secondary); }
.fs-plan-tab:hover { background: var(--accent-subtle); }
.fs-plan-tab-active { border-bottom-color: var(--accent); }
.fs-plan-tab-active .fs-plan-tab-text { color: var(--accent); font-weight: 600; }
.fs-plan-tab-locked { opacity: 0.4; cursor: default; }
.fs-plan-side { padding: 12px 8px; background: var(--surface); border-right-width: 1px; border-right-color: var(--border); gap: 2px; }
.fs-plan-side-heading { font-size: 12px; font-weight: 600; color: var(--text-tertiary); padding: 14px 10px 4px 10px; }
.fs-plan-side-item { gap: 10px; padding: 8px 10px; border-radius: 6px; cursor: pointer; align-items: center; }
.fs-plan-side-item:hover { background: var(--accent-subtle); }
.fs-plan-side-selected { background: var(--selection); }
.fs-plan-side-title { font-size: 15px; }
.fs-plan-side-count { font-size: 13px; color: var(--text-secondary); }
.fs-plan-columns { display: flex; flex-direction: row; gap: 16px; padding: 16px; align-items: flex-start; }
.fs-plan-column { background: var(--accent-subtle); border-radius: 10px; padding: 10px; gap: 8px; }
.fs-plan-column-head { gap: 10px; padding: 6px 6px 4px 6px; align-items: center; }
.fs-plan-column-name { font-size: 15px; font-weight: 600; color: var(--text-secondary); }
.fs-plan-column-count { font-size: 13px; color: var(--text-secondary); }
.fs-plan-cards { display: flex; flex-direction: column; gap: 8px; }
.fs-plan-card { background: var(--surface); border-radius: 8px; padding: 12px; gap: 8px; cursor: pointer; border-width: 2px; border-color: transparent; }
.fs-plan-card:hover { background: var(--surface-elevated); }
.fs-plan-card-selected { border-color: var(--accent); }
.fs-plan-card-title { font-size: 15px; color: var(--text-primary); }
.fs-plan-initials { width: 28px; height: 28px; border-radius: 14px; align-items: center; justify-content: center; margin-left: 2px; }
.fs-plan-initials-text { font-size: 11px; font-weight: 700; color: #FFFFFF; }
.fs-plan-tile-0 { background: #2F7BF5; }
.fs-plan-tile-1 { background: #E8573F; }
.fs-plan-tile-2 { background: #8B5CF6; }
.fs-plan-tile-3 { background: #F2A516; }
.fs-plan-tile-4 { background: #12A58A; }
.fs-plan-tile-5 { background: #E0368F; }
.fs-plan-tile-6 { background: #7BB51C; }
.fs-plan-facts { align-items: center; gap: 4px; }
.fs-plan-create { padding: 8px 8px; border-radius: 6px; cursor: pointer; }
.fs-plan-create:hover { background: var(--surface); }
.fs-plan-create-text { font-size: 15px; color: var(--text-secondary); }
.fs-plan-entry { padding: 4px 0; }
.fs-plan-entry-field { font-size: 15px; }
.fs-plan-panel { background: var(--surface); border-left-width: 1px; border-left-color: var(--border); padding: 16px; gap: 12px; }
.fs-plan-panel-close { align-self: flex-end; padding: 6px 10px; border-radius: 6px; cursor: pointer; }
.fs-plan-panel-close:hover { background: var(--accent-subtle); }
.fs-plan-panel-title { font-size: 22px; font-weight: 600; }
.fs-plan-panel-note { background: var(--warning-bg); border-radius: 6px; padding: 8px 10px; font-size: 14px; }
.fs-plan-detail { gap: 12px; }
.fs-plan-detail-label { font-size: 14px; color: var(--text-secondary); width: 110px; flex-shrink: 0; }
.fs-plan-detail-value { font-size: 14px; }
.fs-plan-moves { gap: 6px; }
.fs-plan-move { padding: 6px 12px; border-radius: 16px; border-width: 1px; border-color: var(--border); cursor: pointer; }
.fs-plan-move:hover { background: var(--accent-subtle); }
.fs-plan-move-here { background: var(--selection); border-color: var(--accent); }
.fs-plan-page { padding: 32px 32px 48px 32px; gap: 8px; }
.fs-plan-big-title { font-size: 30px; font-weight: 700; }
.fs-plan-big-note { font-size: 16px; color: var(--text-secondary); padding-bottom: 8px; }
.fs-plan-section { padding-top: 22px; }
.fs-plan-section-head { border-bottom-width: 1px; border-bottom-color: var(--border); padding-bottom: 6px; margin-bottom: 4px; }
.fs-plan-section-title { font-size: 16px; font-weight: 700; }
.fs-plan-section-title.fs-plan-chip-accent { color: var(--accent); background: transparent; }
.fs-plan-section-title.fs-plan-chip-warning { color: var(--warning); background: transparent; }
.fs-plan-section-title.fs-plan-chip-neutral { color: var(--text-secondary); background: transparent; }
.fs-plan-section-title.fs-plan-chip-success { color: var(--success); background: transparent; }
.fs-plan-section-title.fs-plan-chip-danger { color: var(--error); background: transparent; }
.fs-plan-line { padding: 4px 0; }
.fs-plan-line-head { align-items: flex-start; gap: 12px; }
.fs-plan-line-main { align-items: center; gap: 12px; padding: 6px 8px; border-radius: 8px; cursor: pointer; }
.fs-plan-line-main:hover { background: var(--accent-subtle); }
.fs-plan-line-title { font-size: 17px; }
.fs-plan-line-sub { font-size: 13px; color: var(--text-secondary); }
.fs-plan-line-reason { font-size: 12px; color: var(--text-secondary); }
.fs-plan-line-more { padding: 4px 8px 10px 44px; gap: 6px; }
.fs-plan-check { width: 22px; height: 22px; border-radius: 11px; border-width: 2px; border-color: var(--text-tertiary); margin-top: 8px; flex-shrink: 0; align-items: center; justify-content: center; cursor: pointer; }
.fs-plan-check:hover { border-color: var(--accent); }
.fs-plan-check-on { background: var(--accent); border-color: var(--accent); }
.fs-plan-check-mark { font-size: 13px; color: var(--text-inverse); }
.fs-plan-add { align-items: center; gap: 12px; padding-top: 24px; }
.fs-plan-add-plus { font-size: 24px; color: var(--accent); padding-left: 4px; }
";
