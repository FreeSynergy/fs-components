//! The style sheet of Craft: classes a program puts on its elements to get the look (tiles that share their lines, capital mono
//! labels, square buttons, numbered boxes, the footer). Plain CSS text over theme variables, no Blinc here.
//!
//! Pattern: none (one constant and two pure functions). The classes carry the measures of `CRAFT-MEASUREMENT.md`.

/// The classes. A program adds this to its own sheet; the colours come from the theme variables, so it follows the theme.
pub const SHEET: &str = r#"
.craft-mono { font-family: "JetBrains Mono", "DejaVu Sans Mono", "Noto Sans Mono", monospace; }
.craft-caps { font-family: "JetBrains Mono", "DejaVu Sans Mono", "Noto Sans Mono", monospace; font-size: 11px; font-weight: 700; letter-spacing: 0.06em; color: var(--text-primary); }
.craft-caps-dim { font-family: "JetBrains Mono", "DejaVu Sans Mono", "Noto Sans Mono", monospace; font-size: 11px; font-weight: 400; letter-spacing: 0.08em; color: var(--text-secondary); }
.craft-bar { border-bottom-width: 1px; border-bottom-color: var(--border); background: var(--background); }
.craft-cell { border-left-width: 1px; border-left-color: var(--border); }
.craft-pill { padding: 6px 10px; background: var(--text-primary); }
.craft-pill-text { font-family: "JetBrains Mono", "DejaVu Sans Mono", "Noto Sans Mono", monospace; font-size: 11px; font-weight: 700; letter-spacing: 0.06em; color: var(--background); }
.craft-box { border-width: 1px; border-color: var(--border); }
.craft-tab { padding: 0 14px; height: 34px; border-right-width: 1px; border-right-color: var(--border); cursor: pointer; }
.craft-tab:hover { background: var(--accent-subtle); }
.craft-tab-on { background: var(--text-primary); }
.craft-field { background: var(--input-bg); border-width: 1px; border-color: var(--border); }
.craft-grid { border-top-width: 1px; border-top-color: var(--border); border-left-width: 1px; border-left-color: var(--border); }
.craft-tile { border-right-width: 1px; border-right-color: var(--border); border-bottom-width: 1px; border-bottom-color: var(--border); background: var(--background); }
.craft-tile:hover { background: var(--accent-subtle); }
.craft-tile-head { height: 39px; padding: 0 20px; border-bottom-width: 1px; border-bottom-color: var(--border); }
.craft-tile-body { padding: 20px; }
.craft-number { height: 22px; min-width: 31px; padding: 0 6px; }
.craft-number-text { font-family: "JetBrains Mono", "DejaVu Sans Mono", "Noto Sans Mono", monospace; font-size: 11px; font-weight: 700; color: #000000; }
.craft-badge { padding: 4px 8px; border-width: 1px; border-color: var(--border); background: var(--input-bg); }
.craft-btn { height: 34px; border-width: 1px; border-color: var(--border); cursor: pointer; }
.craft-btn-fill { background: var(--text-primary); border-color: var(--text-primary); }
.craft-btn-fill-text { font-family: "JetBrains Mono", "DejaVu Sans Mono", "Noto Sans Mono", monospace; font-size: 11px; font-weight: 700; letter-spacing: 0.06em; color: var(--background); }
.craft-btn-line { border-color: #6C6B6C; }
.craft-btn-off { opacity: 0.4; cursor: default; }
.craft-progress-track { height: 4px; background: var(--border); }
.craft-section { padding: 0 0 12px 0; }
.craft-footer { height: 33px; padding: 0 20px; border-top-width: 1px; border-top-color: var(--border); background: var(--background); }
.craft-thead { height: 36px; padding: 0 16px; }
.craft-trow { padding: 12px 16px; border-top-width: 1px; border-top-color: var(--border); }
.craft-trow:hover { background: var(--accent-subtle); }
.craft-nav { padding: 20px 12px; border-right-width: 1px; border-right-color: var(--border); background: var(--background); }
.craft-nav-item { padding: 8px 10px; cursor: pointer; }
.craft-nav-item:hover { background: var(--accent-subtle); }
.craft-nav-off { cursor: default; }
.craft-nav-off:hover { background: transparent; }
.craft-note { font-family: "JetBrains Mono", "DejaVu Sans Mono", "Noto Sans Mono", monospace; font-size: 9px; letter-spacing: 0.08em; color: var(--text-tertiary); }
.craft-pill-off { background: transparent; cursor: pointer; }
.craft-pill-off:hover { background: var(--accent-subtle); }
.craft-sub { font-size: 13px; color: var(--text-secondary); }
.craft-link { cursor: pointer; padding: 4px 0 4px 14px; }
.craft-link:hover { background: var(--accent-subtle); }
.craft-mono-text { font-family: "JetBrains Mono", "DejaVu Sans Mono", "Noto Sans Mono", monospace; font-size: 12px; color: var(--text-primary); }
.craft-panel { border-width: 1px; border-color: var(--border); background: var(--background); }
"#;

/// A label as the Craft look wants it: capitals. (The sheet sets the face, size and spacing; the capitals are the text's.)
pub fn caps(label: &str) -> String {
    label.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_label_becomes_capitals_and_umlauts_follow() {
        assert_eq!(caps("Update to 0.5.0"), "UPDATE TO 0.5.0");
        assert_eq!(caps("Größe"), "GRÖSSE");
    }

    #[test]
    fn the_sheet_has_no_rounded_corner() {
        assert!(!SHEET.contains("border-radius"));
        assert!(SHEET.contains(".craft-tile"));
    }
}
