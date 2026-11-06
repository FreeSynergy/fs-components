//! The agenda: the items of the span as a list by day, days without items left out.
//!
//! Pattern: Adapter. A day's heading reports the day; an item reports the item. An item over several
//! days appears on each.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::Scroll;

use super::day::{Day, Moment};
use super::item::{Drawing, items_on};
use super::layout::segment_on;
use super::model::{CalendarItem, Picked};
use crate::list_view::ListPosition;

const HEADING_HEIGHT: f32 = 32.0;
const ROW_HEIGHT: f32 = 30.0;
const TIME_WIDTH: f32 = 116.0;
const PADDING: f32 = 12.0;
const SCROLLBAR_ROOM: f32 = 10.0;

/// The items of `first` to `last`, filling `width` x `height`.
pub(super) fn agenda(
    drawing: &Drawing,
    items: &[CalendarItem],
    (first, last): (Day, Day),
    position: &ListPosition,
    (width, height): (f32, f32),
) -> Div {
    let root = div().class("fs-calendar").flex_col().w(width).h(height);
    let mut column = div()
        .class("fs-calendar-agenda")
        .flex_col()
        .w((width - SCROLLBAR_ROOM).max(0.0))
        .flex_shrink_0();
    let mut any = false;
    let mut day = first;
    while day <= last {
        let on_day = items_on(items, day);
        if !on_day.is_empty() {
            any = true;
            column = column.child(heading(drawing, day, width));
            for item in on_day {
                column = column.child(row(drawing, item, day, width));
            }
        }
        day = day.add_days(1);
    }
    if !any {
        return root.child(
            div()
                .class("fs-calendar-empty")
                .child(text(&drawing.names.empty)),
        );
    }
    root.child(
        Scroll::with_physics(Arc::clone(&position.physics))
            .bind(&position.scroll)
            .w(width)
            .h(height)
            .content(column),
    )
}

fn heading(drawing: &Drawing, day: Day, width: f32) -> Div {
    let element = div()
        .class("fs-calendar-agenda-day")
        .flex_row()
        .items_center()
        .w((width - SCROLLBAR_ROOM).max(0.0))
        .h(HEADING_HEIGHT)
        .child(text(drawing.names.date_text(day)).no_wrap().no_cursor());
    let element = drawing.day_classes(element, day);
    let element = drawing.named(element, &format!("day-{day}"));
    drawing.reports(element, Picked::Day(day))
}

/// When on this day: "all day", or from and to as the clock shows them on this day.
fn time_text(drawing: &Drawing, item: &CalendarItem, day: Day) -> String {
    if item.all_day {
        return drawing.names.all_day.clone();
    }
    let Some(part) = segment_on(item, day) else {
        return String::new();
    };
    let clock = |minute: u16| Moment::at(day, minute / 60, minute % 60).clock();
    if part.end == part.start {
        return clock(part.start);
    }
    let end = if part.end >= super::day::MINUTES_PER_DAY {
        "24:00".to_string()
    } else {
        clock(part.end)
    };
    format!("{}\u{2013}{end}", clock(part.start))
}

fn row(drawing: &Drawing, item: &CalendarItem, day: Day, width: f32) -> Div {
    let room = (width - TIME_WIDTH - 2.0 * PADDING - SCROLLBAR_ROOM).max(0.0);
    div()
        .class("fs-calendar-agenda-row")
        .flex_row()
        .items_center()
        .w((width - SCROLLBAR_ROOM).max(0.0))
        .h(ROW_HEIGHT)
        .child(
            div()
                .class("fs-calendar-agenda-time")
                .w(TIME_WIDTH)
                .flex_shrink_0()
                .child(text(time_text(drawing, item, day)).no_wrap().no_cursor()),
        )
        .child(
            drawing
                .chip(item, day, &item.title, room)
                .w(room)
                .h(ROW_HEIGHT - 4.0),
        )
}
