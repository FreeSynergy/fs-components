//! One tile.
//!
//! Pattern: Adapter — the data is what the caller says; the widget has no words and no
//! colours of its own and does not know what a file is. A tile is a plain element with no click of its
//! own; the grid is what takes the click, so no press reaches two targets.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

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
