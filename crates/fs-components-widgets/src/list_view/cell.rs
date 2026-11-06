//! A text that does not fit its cell.
//!
//! Pattern: pure function over the framework's text measurer — Blinc has no ellipsis of its own, so
//! a long text is cut with one here rather than at the cell's edge. Measured at the size a plain
//! text has; a theme with another size only shifts where the cut falls.

use blinc_layout::text_measure::measure_text;

/// The size a cell's text is measured at.
const FONT_SIZE: f32 = 14.0;
/// Padding of a cell, generously.
const CELL_PADDING: f32 = 36.0;
/// What a mark takes of the first cell, about.
pub(super) const FIRST_CELL_MARK_ROOM: f32 = 80.0;
/// What closes a cut text.
const ELLIPSIS: char = '\u{2026}';

/// `text` as it fits a cell `width` pixels wide: as it is, or cut with an ellipsis.
pub fn fit_cell(text: &str, width: f32) -> String {
    let room = width - CELL_PADDING;
    if measure_text(text, FONT_SIZE).width <= room {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let (mut fits, mut too_long) = (0, chars.len());
    while too_long - fits > 1 {
        let middle = usize::midpoint(fits, too_long);
        let mut candidate: String = chars[..middle].iter().collect();
        candidate.push(ELLIPSIS);
        if measure_text(&candidate, FONT_SIZE).width <= room {
            fits = middle;
        } else {
            too_long = middle;
        }
    }
    let mut cut: String = chars[..fits].iter().collect();
    cut.push(ELLIPSIS);
    cut
}
