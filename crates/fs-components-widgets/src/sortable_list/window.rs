//! Which rows of a long lane are drawn.
//!
//! Pattern: pure function — a lane of 100 000 items draws a bounded window around the focus; the
//! rows before and after are counted in a line, never dropped.

use std::ops::Range;

/// The rows `start..end` to draw of `len`, at most `max`, containing `focus` when there is one.
pub fn window(len: usize, focus: Option<usize>, max: usize) -> Range<usize> {
    let max = max.max(1);
    if len <= max {
        return 0..len;
    }
    let focus = focus.unwrap_or(0).min(len - 1);
    let start = focus.saturating_sub(max / 2).min(len - max);
    start..start + max
}
