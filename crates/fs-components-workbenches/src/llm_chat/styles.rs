//! The style sheet of the chat.
//!
//! Pattern: Composite of templates — the widgets' sheet plus the chat's own, theme variables only.

/// The style sheet of the chat and of the widgets it uses; add it to the theme of the window.
pub fn styles() -> String {
    format!("{}{STYLES}", fs_components_widgets::styles())
}

/// CSS of the chat: only theme variables.
const STYLES: &str = r"
.fs-llm { display: flex; flex-direction: row; background: var(--background); }
.fs-llm-main { display: flex; flex-direction: column; min-width: 0; min-height: 0; }
.fs-llm-side { display: flex; flex-direction: column; flex-shrink: 0; border-left-width: 1px; border-color: var(--border); }
.fs-llm-sidehead, .fs-llm-head { display: flex; flex-direction: row; align-items: center; flex-direction: row; align-items: center; gap: var(--space-3); flex-shrink: 0; padding: 0 var(--space-3); background: var(--surface-elevated); }
.fs-llm-title { color: var(--text-primary); font-size: var(--text-base); }
.fs-llm-meta { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-llm-attention { color: var(--text-primary); font-size: var(--text-sm); padding: 0 var(--space-2); border-radius: var(--radius-sm); background: var(--warning-bg); border-width: 1px; border-color: var(--warning); }
.fs-llm-headtext { display: flex; flex-direction: row; align-items: center; gap: var(--space-3); overflow: hidden; min-width: 0; }
.fs-llm-extras { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); overflow: hidden; min-width: 0; }
.fs-llm-stop { display: flex; flex-direction: row; justify-content: flex-end; }
.fs-llm-stopbox { display: flex; flex-direction: row; justify-content: flex-end; }
.fs-llm-dock { display: flex; overflow: hidden; flex-direction: column; flex-shrink: 0; gap: var(--space-1); padding: var(--space-2) var(--space-3); background: var(--warning-bg); border-top-width: 2px; border-color: var(--warning); }
";
