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
use blinc_layout::text;

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
            let place = align.get(c).copied().unwrap_or_default();
            // A cell that starts with `[ ]` or `[x]` is a checkbox: our extension, GFM has task items in lists only.
            let content = match task_cell(cell) {
                Some((done, rest)) => task_row(done, &rest, face, look, on_link, place),
                None => aligned(cell, face, look, on_link, place),
            };
            let holder = div().class("fs-tv-cell").w(narrowest[c]).child(content);
            line = line.child(sized(holder, portion));
        }
        out = out.child(line);
    }
    out
}

/// Give the cell its share of the row.
fn sized(cell: Div, share: usize) -> Div {
    #[allow(clippy::cast_precision_loss)]
    cell.flex_grow_value(share as f32 * 1.0)
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
    let mark = if done { "\u{2611}" } else { "\u{2610}" };
    row = row.child(
        div()
            .class("fs-tv-marker")
            .child(text(mark).size(face.size)),
    );
    if rest.is_empty() {
        row
    } else {
        row.child(aligned(rest, face, look, on_link, Align::None))
    }
}
