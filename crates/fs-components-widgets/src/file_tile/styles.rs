//! The style sheet of file tile.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the tile and the grid: only theme variables.
pub const STYLES: &str = r"
.fs-tile { padding: var(--space-2); gap: var(--space-1); border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); background: var(--surface); cursor: pointer; }
.fs-tile-plain { border-width: 0px; background: transparent; }
.fs-tile-plain.fs-tile-selected { background: var(--selection); }
.fs-tile:hover { background: var(--accent-subtle); }
.fs-tile-selected { background: var(--selection); border-color: var(--accent); }
.fs-tile-icon { padding: var(--space-2); }
.fs-tile-badge { padding: var(--space-2); border-radius: var(--radius-default); background: var(--surface-elevated); color: var(--text-secondary); font-size: var(--text-sm); }
.fs-tile-name { color: var(--text-primary); font-size: var(--text-sm); }
.fs-tile-line { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-tile-marks { gap: var(--space-1); }
.fs-tile-mark { padding: 0 var(--space-1); border-radius: var(--radius-default); background: var(--warning-bg); color: var(--text-primary); font-size: var(--text-xs); }
.fs-tiles { background: var(--surface); }
.fs-tiles-empty { padding: var(--space-3); color: var(--text-secondary); }
";
