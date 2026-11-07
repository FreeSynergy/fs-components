//! Which messages to build for a scroll position.
//!
//! Pattern: none — arithmetic over [`Heights`], testable without a window.

use std::ops::Range;

use super::heights::Heights;

/// Extra pixels built above and below the view, so a small scroll shows no gap.
pub const OVERSCAN: f32 = 600.0;

/// The messages to build when the list is scrolled to `offset` and `view` pixels high.
pub fn window(heights: &Heights, offset: f32, view: f32) -> Range<usize> {
    let n = heights.len();
    if n == 0 {
        return 0..0;
    }
    let first = heights.at((offset - OVERSCAN).max(0.0));
    let last = heights.at(offset + view + OVERSCAN) + 1;
    first..last.min(n)
}

/// Is what is in view inside what is built (so no rebuild is needed)?
pub fn covered(heights: &Heights, built: &Range<usize>, offset: f32, view: f32) -> bool {
    let n = heights.len();
    let top_ok = built.start == 0 || heights.top(built.start) <= offset;
    let bottom_ok = built.end >= n || heights.top(built.end) >= offset + view;
    top_ok && bottom_ok && !built.is_empty()
}
