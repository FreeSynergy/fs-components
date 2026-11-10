//! The whole grid: toolbar and the body of the mode.
//!
//! Pattern: View function — asks the source for the shown range, then picks the body by mode.

use super::{month, timed, toolbar};
use blinc_layout::div::{Div, div};

use crate::handle::CalendarGrid;
use crate::model::Mode;

/// The whole grid (toolbar and body) in `width` x `height`, built from the state now.
pub fn grid(g: &CalendarGrid, width: f32, height: f32) -> Div {
    let state = g.snapshot();
    let days = state.days();
    let (first, last) = (days[0], days[days.len() - 1]);
    let items = g.source.items(first, last);
    let bar = g.look.style.heights.control + g.look.style.spacing.s;
    let body_h = (height - bar).max(0.0);
    let body = match state.mode {
        Mode::Month => month::month(g, &items, width, body_h),
        Mode::Week | Mode::Day => timed::timed(g, &items, width, body_h),
    };
    div()
        .flex_col()
        .w(width)
        .h(height)
        .child(toolbar::toolbar(g))
        .child(body)
}
