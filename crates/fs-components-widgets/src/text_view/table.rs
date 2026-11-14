//! A table.
//!
//! Pattern: Adapter — rows of cells; a column's share of the width follows the longest text in
//! it (bounded), because Blinc has no table layout with automatic column widths.

use blinc_layout::div::{Div, div};

use super::LinkHandler;
use super::inline::{Face, aligned};
use super::look::Look;
use super::task::task_cell;
use crate::text::{Align, Inline};

/// The room a cell keeps around its text on both sides, in pixels (`--space-2` twice).
const CELL_PAD: f32 = 16.0;
/// A column asks for at least and at most this many pixels of text; the room is shared in that proportion.
const LEAST: f32 = 60.0;
const MOST: f32 = 360.0;

/// The width of `text` on one line as it is drawn (bold in the head row), measured with the font, not estimated.
fn measured(text: &str, size: f32, bold: bool) -> f32 {
    let face = blinc_layout::text_measure::TextFace {
        weight: if bold { 700 } else { 400 },
        ..Default::default()
    };
    text.split('\n')
        .map(|line| face.measure(line, size).width)
        .fold(0.0, f32::max)
}

/// The element of a table.
pub(super) fn table(
    align: &[Align],
    rows: &[Vec<Inline>],
    look: &Look,
    on_link: Option<&LinkHandler>,
) -> Div {
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    // A column's share of the row follows the width of its longest cell on one line (measured, bounded). The share is a
    // fixed percentage of the row, so every cell has a definite width when its text is laid out: text measured against a
    // width that a flex item only grows into afterwards wraps too early (the finding W1).
    let want: Vec<f32> = (0..columns)
        .map(|c| {
            rows.iter()
                .enumerate()
                .filter_map(|(r, row)| {
                    row.get(c)
                        .map(|cell| measured(&cell.text, look.small, r == 0))
                })
                .fold(0.0, f32::max)
                .clamp(LEAST, MOST)
                + CELL_PAD
        })
        .collect();
    let mut out = div().class("fs-tv-block").class("fs-tv-table").flex_col();
    for (r, row) in rows.iter().enumerate() {
        let mut line = div().class("fs-tv-row");
        if r == 0 {
            line = line.class("fs-tv-head");
        }
        for (c, &portion) in want.iter().enumerate() {
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
            let place = align.get(c).copied().unwrap_or_default();
            // A cell that starts with `[ ]` or `[x]` is a checkbox: our extension, GFM has task items in lists only.
            let content = match task_cell(cell) {
                Some((done, rest)) => task_row(done, &rest, face, look, on_link, place),
                None => aligned(cell, face, look, on_link, place),
            };
            line = line.child(sized(div().class("fs-tv-cell").child(content), portion));
        }
        out = out.child(line);
    }
    out
}

/// Give the cell its share of the row: basis 0 and a growth in the proportion of the shares, so the cell is exactly its
/// share of the row wide and the text in it is laid out at that width.
fn sized(cell: Div, share: f32) -> Div {
    cell.flex_1().flex_grow_value(share).min_w(0.0)
}

/// A checkbox and the text after it, placed by `place`.
fn task_row(
    done: bool,
    rest: &Inline,
    face: Face,
    look: &Look,
    on_link: Option<&LinkHandler>,
    place: Align,
) -> Div {
    let mut row = div().flex_row().items_center().gap(4.0).w_full();
    row = match place {
        Align::Center => row.justify_center(),
        Align::Right => row.justify_end(),
        _ => row,
    };
    row = row.child(super::task::marker(look, done, face.size));
    if rest.is_empty() {
        row
    } else {
        row.child(aligned(rest, face, look, on_link, Align::None))
    }
}
