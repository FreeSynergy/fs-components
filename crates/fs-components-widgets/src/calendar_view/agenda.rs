//! The agenda: the items of the span as a list by day, days without items left out.
//!
//! Pattern: Adapter. A day's heading reports the day; an item reports the item. An item over
//! several days appears on each.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::Scroll;

use super::day::{Day, Moment};
use super::item::{Drawing, places_on};
use super::layout::segment_on;
use super::model::{CalendarItem, Picked};
use crate::list_view::ListPosition;

/// Height of a heading and of an item, the same, so the place of a line is a multiplication.
const LINE_HEIGHT: f32 = 30.0;
const TIME_WIDTH: f32 = 116.0;
const PADDING: f32 = 12.0;
const SCROLLBAR_ROOM: f32 = 10.0;

/// One line of the list.
#[derive(Clone, Copy)]
enum Line {
    /// A day's heading.
    Heading(Day),
    /// An item, by its place in the items, on a day.
    Item(usize, Day),
}

/// The lines of `first` to `last`: only those in view become elements, so a long agenda costs what a
/// short one costs (the way `list-view` does it).
fn lines_of(items: &[CalendarItem], first: Day, last: Day) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut day = first;
    while day <= last {
        let places = places_on(items, day);
        if !places.is_empty() {
            lines.push(Line::Heading(day));
            lines.extend(places.into_iter().map(|place| Line::Item(place, day)));
        }
        day = day.add_days(1);
    }
    lines
}

/// The items of `first` to `last`, filling `width` x `height`.
#[allow(clippy::cast_precision_loss)]
pub(super) fn agenda(
    drawing: &Drawing,
    items: &Arc<Vec<CalendarItem>>,
    (first, last): (Day, Day),
    position: &ListPosition,
    (width, height): (f32, f32),
) -> Div {
    let root = div().class("fs-calendar").flex_col().w(width).h(height);
    let lines = lines_of(items, first, last);
    if lines.is_empty() {
        return root.child(
            div()
                .class("fs-calendar-empty")
                .child(text(&drawing.names.empty)),
        );
    }
    let total = lines.len() as f32 * LINE_HEIGHT;
    let offset = position.offset().clamp(0.0, (total - height).max(0.0));
    position.apply_offset(offset);
    let window = position.window(offset, (height, LINE_HEIGHT, lines.len()));
    let inner = (width - SCROLLBAR_ROOM - 2.0 * PADDING).max(0.0);
    let (drawing, items) = (drawing.clone(), Arc::clone(items));
    let column = div()
        .class("fs-calendar-agenda")
        .relative()
        .w((width - SCROLLBAR_ROOM).max(0.0))
        .h(total)
        .flex_shrink_0()
        .for_each(
            &window,
            |index: &usize| *index,
            move |index: usize| {
                // The window is shared across builds: a build that is still alive may be asked for
                // a line a later build has more of. Beyond what this build has, there is nothing.
                let element = match lines.get(index) {
                    Some(&Line::Heading(day)) => heading(&drawing, day, inner),
                    Some(&Line::Item(item, day)) => match items.get(item) {
                        Some(found) => row(&drawing, found, day, inner),
                        None => div(),
                    },
                    None => div(),
                };
                element
                    .absolute()
                    .left(PADDING)
                    .top(index as f32 * LINE_HEIGHT)
            },
        );
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
        .w(width)
        .h(LINE_HEIGHT)
        .child(text(drawing.names.date_text(day)).no_wrap().no_cursor());
    let element = if drawing.today == Some(day) {
        element.class("fs-calendar-agenda-today")
    } else {
        element
    };
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
    let room = (width - TIME_WIDTH).max(0.0);
    div()
        .class("fs-calendar-agenda-row")
        .flex_row()
        .items_center()
        .w(width)
        .h(LINE_HEIGHT)
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
                .h(LINE_HEIGHT - 4.0),
        )
}
