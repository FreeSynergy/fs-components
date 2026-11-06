//! One day's column on the time axis.
//!
//! Pattern: Adapter (minutes and lanes to positions). The column is a click area with the hour lines,
//! and the items lie beside it, not inside it, so that clicking an item does not report the day too.
//! Overlapping items share the width; where lanes would be too narrow to read, the last ones are
//! replaced by a count.

use blinc_layout::div::{Div, div};

use super::day::{Day, MINUTES_PER_DAY, Moment};
use super::item::Drawing;
use super::layout::{Placed, Segment, lanes, segment_on};
use super::model::{CalendarItem, Picked};

/// One hour in pixels (the docs ask for at least 44).
pub(super) const HOUR_HEIGHT: f32 = 44.0;
/// The gap between chips.
pub(super) const ALL_DAY_ROW_GAP: f32 = 2.0;
const ITEM_GAP: f32 = ALL_DAY_ROW_GAP;
const SHORTEST_ITEM: f32 = 22.0;
/// A lane narrower than this is not drawn; the count on top says what is missing.
const NARROWEST_LANE: f32 = 56.0;

/// Where a minute of the day lies on the axis.
pub(super) fn y_of(minute: u16) -> f32 {
    f32::from(minute) / 60.0 * HOUR_HEIGHT
}

/// One day's column: the click area with its hour lines, then the items in their lanes.
#[allow(clippy::cast_precision_loss)]
pub(super) fn day_column(
    drawing: &Drawing,
    items: &[&CalendarItem],
    day: Day,
    width: f32,
    now: Option<Moment>,
) -> Div {
    let total = y_of(MINUTES_PER_DAY);
    let mut area = div()
        .class("fs-calendar-column")
        .absolute()
        .left(0.0)
        .top(0.0)
        .w(width)
        .h(total)
        .flex_col();
    for _ in 0..24 {
        area = area.child(div().class("fs-calendar-hour").w(width).h(HOUR_HEIGHT));
    }
    let area = drawing.named(area, &format!("day-{day}"));
    let area = drawing.reports(area, Picked::Day(day));
    let mut column = div()
        .relative()
        .w(width)
        .h(total)
        .flex_shrink_0()
        .child(area);
    let segments: Vec<Segment> = items
        .iter()
        .filter_map(|item| segment_on(item, day))
        .collect();
    let shortest = (SHORTEST_ITEM / HOUR_HEIGHT * 60.0).ceil();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let placed = lanes(&segments, shortest as u16);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let room = ((width / NARROWEST_LANE) as usize).max(1);
    let mut hidden: Vec<u16> = Vec::new();
    for place in &placed {
        let (item, segment) = (items[place.index], segments[place.index]);
        if place.lane >= room {
            hidden.push(segment.start);
            continue;
        }
        column = column.child(placed_item(
            drawing,
            item,
            segment,
            day,
            place,
            (width, room),
        ));
    }
    if !hidden.is_empty() {
        let first = hidden.iter().copied().min().unwrap_or(0);
        let top = y_of(first).min(total - SHORTEST_ITEM);
        column = column.child(
            drawing
                .more(hidden.len(), day)
                .absolute()
                .left((width - NARROWEST_LANE).max(0.0))
                .top(top)
                .w(NARROWEST_LANE.min(width))
                .h(SHORTEST_ITEM),
        );
    }
    if let Some(now) = now.filter(|n| n.day == day) {
        column = column.child(
            div()
                .class("fs-calendar-now")
                .absolute()
                .left(0.0)
                .top(y_of(now.minute))
                .w(width)
                .h(2.0),
        );
    }
    column
}

#[allow(clippy::cast_precision_loss)]
fn placed_item(
    drawing: &Drawing,
    item: &CalendarItem,
    segment: Segment,
    day: Day,
    place: &Placed,
    (width, room): (f32, usize),
) -> Div {
    let total = y_of(MINUTES_PER_DAY);
    let height = (y_of(segment.end) - y_of(segment.start)).max(SHORTEST_ITEM);
    let top = y_of(segment.start).min(total - SHORTEST_ITEM);
    let lane_width = width / place.lanes.min(room) as f32;
    let shown = lane_width - ITEM_GAP;
    // The axis shows the time; what is scarce in a narrow lane is the title.
    drawing
        .chip(item, day, &item.title, shown)
        .absolute()
        .left(place.lane as f32 * lane_width)
        .top(top)
        .w(shown.max(0.0))
        .h(height - 1.0)
}
