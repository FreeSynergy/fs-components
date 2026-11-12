//! The style sheet of the wiki layouts.
//!
//! Pattern: none — theme variables and a few neutral values; one block per layout so a layout can change without the other.

pub(crate) fn styles() -> String {
    [SHARED, FRAME_A, FRAME_B].concat()
}

const SHARED: &str = r"
.fs-wk-root { display: flex; background: #ffffff; color: var(--text-primary); font-family: var(--font-sans); }
.fs-wk-row { height: 30px; gap: 6px; padding-right: 8px; border-radius: 6px; cursor: pointer; }
.fs-wk-row:hover { background: var(--accent-subtle); }
.fs-wk-row-here { background: var(--selection); color: var(--accent); }
.fs-wk-folder { font-weight: 600; }
.fs-wk-panel { padding: 16px; gap: 6px; }
.fs-wk-panel-title { font-size: 12px; font-weight: 700; color: var(--text-secondary); padding-bottom: 4px; }
.fs-wk-quiet { color: var(--text-secondary); font-size: 13px; }
.fs-wk-outline-row { padding: 3px 6px; border-radius: 4px; font-size: 13px; color: var(--text-secondary); cursor: pointer; }
.fs-wk-outline-row:hover { background: var(--accent-subtle); color: var(--text-primary); }
.fs-wk-rev-row { padding: 4px 6px; border-radius: 4px; font-size: 13px; }
.fs-wk-rev-current { background: var(--selection); }
.fs-wk-langs { gap: 4px; }
.fs-wk-lang { padding: 2px 10px; border-radius: 12px; font-size: 13px; cursor: pointer; color: var(--text-secondary); }
.fs-wk-lang:hover { background: var(--accent-subtle); }
.fs-wk-lang-on { background: var(--selection); color: var(--accent); font-weight: 600; }
.fs-wk-state { gap: 6px; font-size: 13px; color: var(--text-secondary); }
.fs-wk-dot { width: 9px; height: 9px; border-radius: 5px; }
.fs-wk-dot-ok { background: #2f9e6e; }
.fs-wk-dot-old { background: #d9822b; }
.fs-wk-dot-other { background: #3b7dd8; }
.fs-wk-band { gap: 10px; padding: 10px 14px; border-radius: 6px; background: var(--warning-bg); font-size: 14px; }
.fs-wk-crumbs { gap: 6px; font-size: 13px; color: var(--text-secondary); }
.fs-wk-crumb-last { color: var(--text-primary); font-weight: 600; }
.fs-wk-btn { padding: 5px 12px; border-radius: 6px; cursor: pointer; font-size: 13px; gap: 6px; }
.fs-wk-btn:hover { background: var(--accent-subtle); }
.fs-wk-tick { height: 2px; border-radius: 1px; background: var(--border); }
.fs-wk-block { padding: 3px 6px; border-radius: 4px; }
.fs-wk-block:hover { background: var(--accent-subtle); }
.fs-wk-block-editing { background: #ffffff; }
.fs-wk-grip { width: 18px; color: var(--text-secondary); font-size: 14px; }
";

const FRAME_A: &str = r"
.fs-wk-a-top { height: 52px; padding: 0 16px; gap: 16px; border-bottom-width: 1px; border-bottom-color: var(--border); background: var(--surface); }
.fs-wk-a-brand { font-weight: 700; font-size: 16px; gap: 8px; width: 268px; }
.fs-wk-a-side { width: 300px; background: #f7f8f9; border-right-width: 1px; border-right-color: var(--border); }
.fs-wk-a-space { padding: 14px 16px 6px 16px; gap: 2px; }
.fs-wk-a-space-name { font-weight: 700; font-size: 15px; }
.fs-wk-a-crumbs { height: 44px; padding: 0 28px; }
.fs-wk-a-page { padding: 8px 40px 48px 40px; gap: 14px; max-width: 880px; }
.fs-wk-a-title { font-size: 30px; font-weight: 700; line-height: 1.25; }
.fs-wk-a-meta { gap: 18px; padding-bottom: 6px; }
.fs-wk-a-right { width: 270px; border-left-width: 1px; border-left-color: var(--border); background: var(--surface); }
.fs-wk-a-sep { height: 1px; background: var(--border); }
.fs-wk-row-a-here { border-left-width: 3px; border-left-color: var(--accent); }
";

const FRAME_B: &str = r"
.fs-wk-b-side { width: 264px; background: #f7f6f3; }
.fs-wk-b-head { padding: 14px 14px 6px 14px; gap: 10px; }
.fs-wk-b-section { padding: 14px 16px 4px 16px; }
.fs-wk-b-section-text { font-size: 12px; font-weight: 600; color: var(--text-secondary); }
.fs-wk-row-quiet { height: 28px; border-radius: 5px; font-size: 14px; color: var(--text-secondary); }
.fs-wk-row-quiet.fs-wk-row-here { background: #ebe9e4; color: var(--text-primary); }
.fs-wk-b-top { height: 44px; padding: 0 16px; gap: 12px; }
.fs-wk-b-page { padding: 72px 24px 120px 24px; gap: 14px; }
.fs-wk-b-title { font-size: 44px; font-weight: 800; line-height: 1.2; }
.fs-wk-b-meta { gap: 14px; padding-bottom: 8px; }
.fs-wk-ticks { width: 40px; padding: 120px 10px 0 0; gap: 9px; }
";
