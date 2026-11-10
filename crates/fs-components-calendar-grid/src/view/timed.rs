//! The week grid and the day column: header, all-day row, and the scrolling time axis.
//!
//! Pattern: View function — this file works out the measures ([`Geo`]) and stacks the three
//! parts; `head.rs` draws the header and the all-day row, `body.rs` the time axis. A column is one
//! click target and works out the minute from the pointer (48 slots times 7 days would be 336 nodes
//! per build); items are a layer above the columns, not their children, so a click on an item
//! cannot also select the slot under it.

use blinc_layout::div::{Div, div};
use blinc_layout::widgets::scroll::scroll;

use super::{body, head};
use crate::handle::CalendarGrid;
use crate::model::GridItem;
use crate::spans::{SpanBar, lanes_used, span_rows};

/// Width of the hour labels at the left, in pixels.
pub const LABEL_W: f32 = 56.0;

/// The measures of one build.
pub struct Geo {
    /// Total width.
    pub width: f32,
    /// Width of one day column.
    pub col: f32,
    /// Height of the header row.
    pub head: f32,
    /// Height of one bar.
    pub bar: f32,
    /// Height of the all-day row.
    pub allday: f32,
    /// Pixels per hour on the axis.
    pub hour: f32,
    /// Bars of the all-day row.
    pub bars: Vec<SpanBar>,
}

/// The timed grid filling `width` x `height` (seven columns, or one in day mode).
pub fn timed(grid: &CalendarGrid, items: &[GridItem], width: f32, height: f32) -> Div {
    let state = grid.snapshot();
    let s = grid.look.style;
    let days = state.days();
    let bars = span_rows(items, &days, days.len(), true)
        .into_iter()
        .next()
        .unwrap_or_default();
    let bar = (s.text.caption * 1.6).ceil();
    let geo = Geo {
        width,
        col: (width - LABEL_W) / days.len() as f32,
        head: s.heights.row,
        bar,
        allday: lanes_used(&bars).max(1) as f32 * (bar + 2.0) + 4.0,
        hour: (s.heights.row * 1.5).round(),
        bars,
    };
    let avail = (height - geo.head - geo.allday).max(geo.hour);
    div()
        .flex_col()
        .w(width)
        .h(height)
        .id("cal-timed")
        .child(head::header(grid, &state, &days, &geo))
        .child(head::allday(grid, &state, &days, items, &geo))
        .child(
            scroll()
                .w(width)
                .h(avail)
                .content(body::body(grid, &state, &days, items, &geo)),
        )
}
