//! The month: six weeks of days, each with as many items as fit and a count of the rest.
//!
//! Pattern: Adapter (positions from the sizes to elements). Each day is a click area with its number,
//! and the items lie beside it, not inside it, so that clicking an item does not report the day too.

use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::day::Day;
use super::item::{Drawing, items_on};
use super::layout::month_days;
use super::model::{CalendarItem, Picked};

const HEAD_HEIGHT: f32 = 28.0;
/// Room of the day's number.
const NUMBER_HEIGHT: f32 = 24.0;
const CHIP_HEIGHT: f32 = 20.0;
const CHIP_GAP: f32 = 2.0;
const CHIP_MARGIN: f32 = 2.0;
/// Width of the day's number.
const NUMBER_WIDTH: f32 = 24.0;
/// Below this width a chip shows the title without the time.
const WIDE_FOR_TIME: f32 = 110.0;

/// The month of `anchor`, filling `width` x `height`.
#[allow(clippy::cast_precision_loss)]
pub(super) fn month(
    drawing: &Drawing,
    items: &[CalendarItem],
    anchor: Day,
    week_start: u8,
    (width, height): (f32, f32),
) -> Div {
    let cell_width = width / 7.0;
    let cell_height = ((height - HEAD_HEIGHT) / 6.0).max(0.0);
    let days = month_days(anchor, week_start);
    let mut head = div()
        .class("fs-calendar-head")
        .flex_row()
        .w(width)
        .h(HEAD_HEIGHT);
    for day in days.iter().take(7) {
        let name = &drawing.names.weekdays[usize::from(day.weekday())];
        head = head.child(
            div()
                .class("fs-calendar-head-cell")
                .flex_row()
                .items_center()
                .w(cell_width)
                .h(HEAD_HEIGHT)
                .child(text(name).no_wrap().no_cursor()),
        );
    }
    let mut root = div()
        .class("fs-calendar")
        .flex_col()
        .w(width)
        .h(height)
        .child(head);
    for week in days.chunks(7) {
        let mut row = div().flex_row().w(width).h(cell_height);
        for day in week {
            let inside = day.month() == anchor.month();
            let on_day = items_on(items, *day);
            row = row.child(cell(
                drawing,
                &on_day,
                *day,
                inside,
                (cell_width, cell_height),
            ));
        }
        root = root.child(row);
    }
    root
}

/// How many chips a cell of `height` holds of `count`, and how many are left for the count.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn fitting(count: usize, height: f32) -> (usize, usize) {
    let room = ((height - NUMBER_HEIGHT) / (CHIP_HEIGHT + CHIP_GAP)).max(0.0) as usize;
    if count <= room {
        (count, 0)
    } else {
        let shown = room.saturating_sub(1);
        (shown, count - shown)
    }
}

#[allow(clippy::cast_precision_loss)]
fn cell(
    drawing: &Drawing,
    items: &[&CalendarItem],
    day: Day,
    inside: bool,
    (width, height): (f32, f32),
) -> Div {
    let number = div()
        .class("fs-calendar-number")
        .w(NUMBER_WIDTH)
        .flex_row()
        .items_center()
        .justify_center()
        .child(text(day.day().to_string()).no_wrap().no_cursor());
    let area = div()
        .class("fs-calendar-day")
        .absolute()
        .left(0.0)
        .top(0.0)
        .w(width)
        .h(height)
        .flex_col()
        .child(number);
    let area = if inside {
        area
    } else {
        area.class("fs-calendar-outside")
    };
    let area = drawing.day_classes(area, day);
    let area = drawing.named(area, &format!("day-{day}"));
    let area = drawing.reports(area, Picked::Day(day));
    let mut holder = div().relative().w(width).h(height).child(area);
    let chip_width = (width - 2.0 * CHIP_MARGIN).max(0.0);
    let (shown, left) = fitting(items.len(), height);
    let mut top = NUMBER_HEIGHT;
    for item in items.iter().take(shown) {
        // Where a mark takes room, or there is little, the title is what must be read.
        let plain = item.all_day
            || item.start.day != day
            || chip_width < WIDE_FOR_TIME
            || !item.marks.is_empty();
        let label = if plain {
            item.title.clone()
        } else {
            format!("{} {}", item.start.clock(), item.title)
        };
        holder = holder.child(
            drawing
                .chip(item, day, &label, chip_width)
                .absolute()
                .left(CHIP_MARGIN)
                .top(top)
                .w(chip_width)
                .h(CHIP_HEIGHT),
        );
        top += CHIP_HEIGHT + CHIP_GAP;
    }
    if left > 0 {
        holder = holder.child(
            drawing
                .more(left, day)
                .absolute()
                .left(CHIP_MARGIN)
                .top(top)
                .w(chip_width)
                .h(CHIP_HEIGHT),
        );
    }
    holder
}
