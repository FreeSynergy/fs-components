//! A table.
//!
//! Pattern: Adapter — rows of cells; a column's share of the width follows the longest text in
//! it (bounded), because Blinc has no table layout with automatic column widths.

use blinc_layout::div::{Div, div};

use super::LinkHandler;
use super::inline::{Face, inline};
use super::look::Look;
use crate::text::{Align, Inline};

/// Longest word (in characters) a column keeps whole.
const MAX_WORD: usize = 28;
/// The widest a column asks to start with, in pixels: wider tables shrink to the room there is.
const MAX_BASIS: f32 = 160.0;
const MIN_SHARE: usize = 6;
const MAX_SHARE: usize = 40;

/// The element of a table.
pub(super) fn table(
    align: &[Align],
    rows: &[Vec<Inline>],
    look: &Look,
    on_link: Option<&LinkHandler>,
) -> Div {
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    let share: Vec<usize> = (0..columns)
        .map(|c| {
            rows.iter()
                .filter_map(|r| r.get(c))
                .map(|cell| cell.text.chars().count())
                .max()
                .unwrap_or(0)
                .clamp(MIN_SHARE, MAX_SHARE)
        })
        .collect();
    let narrowest: Vec<f32> = (0..columns)
        .map(|c| {
            let word = rows
                .iter()
                .filter_map(|r| r.get(c))
                .flat_map(|cell| cell.text.split_whitespace())
                .map(|w| w.chars().count())
                .max()
                .unwrap_or(0);
            #[allow(clippy::cast_precision_loss)]
            let chars = word.min(MAX_WORD) as f32;
            (chars * look.small * 0.7 + 2.0 * look.small).min(MAX_BASIS)
        })
        .collect();
    let mut out = div().class("fs-tv-block").class("fs-tv-table").flex_col();
    for (r, row) in rows.iter().enumerate() {
        let mut line = div().class("fs-tv-row");
        if r == 0 {
            line = line.class("fs-tv-head");
        }
        for (c, &portion) in share.iter().enumerate() {
            let empty = Inline::default();
            let cell = row.get(c).unwrap_or(&empty);
            let face = Face {
                size: look.small,
                bold: r == 0,
                mono: false,
                cell: true,
            };
            // A column starts as wide as its longest word (at most `MAX_BASIS`), grows by its share
            // of the room, and shrinks when there is not enough: a table wider than the window
            // wraps words inside their columns (also in the middle of a long word) and never
            // runs past the edge, so no column is cut off.
            let holder = div()
                .class("fs-tv-cell")
                .w(narrowest[c])
                .child(inline(cell, face, look, on_link));
            line = line.child(sized(holder, portion, align.get(c).copied()));
        }
        out = out.child(line);
    }
    out
}

/// Give the cell its share of the row.
fn sized(cell: Div, share: usize, _align: Option<Align>) -> Div {
    #[allow(clippy::cast_precision_loss)]
    cell.flex_grow_value(share as f32 * 1.0)
}
