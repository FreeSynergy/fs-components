//! `file-tile`: one thing as a tile, and a grid of tiles of which only those in view exist.
//!
//! Pattern: Value object + Adapter — the data is what the caller says; the widget has no words and no
//! colours of its own and does not know what a file is. A tile is a plain element with no click of its
//! own; [`TileGrid`] is what takes the click, so no press reaches two targets.

mod grid;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

pub use grid::{TileGrid, grid_target};

use crate::list_view::fit_cell;

/// One tile.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileTile {
    /// Handed back when the tile is chosen.
    pub key: String,
    /// The name; a long one is cut with an ellipsis at its end.
    pub name: String,
    /// A short word for what it is, where a picture would be.
    pub badge: String,
    /// Smaller lines below the name, in the caller's words.
    pub lines: Vec<String>,
    /// Markings that must be seen at once, in the caller's words.
    pub marks: Vec<String>,
}

impl FileTile {
    /// A tile.
    pub fn new(key: impl Into<String>, name: impl Into<String>, badge: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            badge: badge.into(),
            ..Self::default()
        }
    }

    /// With these lines below the name.
    #[must_use]
    pub fn with_lines(mut self, lines: Vec<String>) -> Self {
        self.lines = lines;
        self
    }

    /// With these markings.
    #[must_use]
    pub fn with_marks(mut self, marks: Vec<String>) -> Self {
        self.marks = marks;
        self
    }

    /// The tile, `width` x `height`.
    pub fn build(&self, width: f32, height: f32, selected: bool) -> Div {
        let mut tile = div()
            .class("fs-tile")
            .flex_col()
            .w(width)
            .h(height)
            .overflow_clip();
        if selected {
            tile = tile.class("fs-tile-selected");
        }
        tile = tile.child(
            div().class("fs-tile-badge").child(
                text(fit_cell(&self.badge, width))
                    .no_wrap()
                    .no_cursor()
                    .pointer_events_none(),
            ),
        );
        if !self.marks.is_empty() {
            let mut row = div().class("fs-tile-marks").flex_row().flex_wrap();
            for mark in &self.marks {
                row = row.child(
                    div()
                        .class("fs-tile-mark")
                        .child(text(fit_cell(mark, width)).no_wrap().no_cursor()),
                );
            }
            tile = tile.child(row);
        }
        tile = tile.child(
            text(fit_cell(&self.name, width))
                .class("fs-tile-name")
                .no_wrap()
                .no_cursor(),
        );
        for line in &self.lines {
            tile = tile.child(
                text(fit_cell(line, width))
                    .class("fs-tile-line")
                    .no_wrap()
                    .no_cursor(),
            );
        }
        tile
    }
}

/// CSS of the tile and the grid: only theme variables.
pub const STYLES: &str = r"
.fs-tile { padding: var(--space-2); gap: var(--space-1); border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); background: var(--surface); cursor: pointer; }
.fs-tile:hover { background: var(--accent-subtle); }
.fs-tile-selected { background: var(--selection); border-color: var(--accent); }
.fs-tile-badge { padding: var(--space-2); border-radius: var(--radius-default); background: var(--surface-elevated); color: var(--text-secondary); font-size: var(--text-sm); }
.fs-tile-name { color: var(--text-primary); font-size: var(--text-sm); }
.fs-tile-line { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-tile-marks { gap: var(--space-1); }
.fs-tile-mark { padding: 0 var(--space-1); border-radius: var(--radius-default); background: var(--warning-bg); color: var(--text-primary); font-size: var(--text-xs); }
.fs-tiles { background: var(--surface); }
.fs-tiles-empty { padding: var(--space-3); color: var(--text-secondary); }
";
