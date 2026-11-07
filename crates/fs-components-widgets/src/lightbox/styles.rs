//! The style sheet of lightbox.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the lightbox: only theme variables.
pub const STYLES: &str = r"
.fs-lb { display: flex; flex-direction: column; width: 100%; height: 100%; background: var(--surface-overlay); padding: var(--space-3); gap: var(--space-3); }
.fs-lb-bar { display: flex; flex-direction: row; align-items: center; justify-content: space-between; gap: var(--space-3); }
.fs-lb-count { color: var(--text-primary); }
.fs-lb-body { display: flex; flex-direction: row; align-items: center; flex-grow: 1; min-height: 0; gap: var(--space-3); }
.fs-lb-stage { display: flex; flex-direction: column; align-items: center; justify-content: center; flex-grow: 1; flex-basis: 0; min-width: 0; min-height: 0; height: 100%; overflow: hidden; }
.fs-lb-note { color: var(--text-secondary); padding: var(--space-3); }
.fs-lb-caption { color: var(--text-primary); text-align: center; }
";
