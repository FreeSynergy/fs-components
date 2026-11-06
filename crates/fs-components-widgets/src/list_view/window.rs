//! Which rows are in view, and how wide the columns are.
//!
//! Pattern: pure functions — the arithmetic of the list, testable without a window.

use std::ops::Range;

use super::model::{Column, ColumnWidth};

/// The height of a row unless the caller says otherwise, in pixels.
pub const DEFAULT_ROW_HEIGHT: f32 = 32.0;

/// Rows built on either side of those in view, so a small scroll shows no gap.
const OVERSCAN: usize = 4;

/// The rows to build when the list is scrolled to `offset` pixels and `view` pixels high.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn visible_rows(offset: f32, view: f32, row_height: f32, count: usize) -> Range<usize> {
    let row_height = row_height.max(1.0);
    let offset = offset.max(0.0);
    let first = ((offset / row_height) as usize).min(count);
    let last = (((offset + view.max(0.0)) / row_height).ceil() as usize)
        .saturating_add(OVERSCAN)
        .min(count);
    first.saturating_sub(OVERSCAN)..last
}

/// The width of each column in pixels: the fixed ones as asked, the rest share what is left.
pub fn column_widths(columns: &[Column], width: f32) -> Vec<f32> {
    let fixed: f32 = columns
        .iter()
        .map(|c| match c.width {
            ColumnWidth::Pixels(px) => px.max(0.0),
            ColumnWidth::Share(_) => 0.0,
        })
        .sum();
    let shares: f32 = columns
        .iter()
        .map(|c| match c.width {
            ColumnWidth::Share(s) => s.max(0.0),
            ColumnWidth::Pixels(_) => 0.0,
        })
        .sum();
    let left = (width - fixed).max(0.0);
    columns
        .iter()
        .map(|c| match c.width {
            ColumnWidth::Pixels(px) => px.max(0.0),
            ColumnWidth::Share(s) if shares > 0.0 => left * s.max(0.0) / shares,
            ColumnWidth::Share(_) => 0.0,
        })
        .collect()
}
