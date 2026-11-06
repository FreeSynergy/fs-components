//! A week or a day on a time axis.
//!
//! Pattern: Adapter (minutes and lanes to positions). A column is a click area with the hour lines,
//! and the items lie beside it, not inside it, so that clicking an item does not report the day too.
//! The axis uses wall-clock minutes: on the day clocks change an hour is missing or repeated, and
//! what happens in it is drawn at the time its clock shows.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::Scroll;

use super::column::HOUR_HEIGHT;
use super::column::{ALL_DAY_ROW_GAP as ITEM_GAP, day_column, y_of};
use super::day::{Day, MINUTES_PER_DAY, Moment};
use super::item::{Drawing, items_on};
use super::model::{CalendarItem, Picked};
use crate::list_view::ListPosition;

const GUTTER: f32 = 56.0;
const HEAD_HEIGHT: f32 = 44.0;
const ALL_DAY_ROW: f32 = 22.0;
const ALL_DAY_ROWS: usize = 3;
const SCROLLBAR_ROOM: f32 = 10.0;

/// What the axis shows besides the days.
pub(super) struct Axis<'a> {
    pub position: &'a ListPosition,
    pub now: Option<Moment>,
}

/// The `days` on the axis, filling `width` x `height`.
#[allow(clippy::cast_precision_loss)]
pub(super) fn timeline(
    drawing: &Drawing,
    items: &[CalendarItem],
    days: &[Day],
    axis: &Axis<'_>,
    (width, height): (f32, f32),
) -> Div {
    let column = ((width - GUTTER - SCROLLBAR_ROOM) / days.len().max(1) as f32).max(0.0);
    let all_day: Vec<Vec<&CalendarItem>> = days
        .iter()
        .map(|day| {
            items_on(items, *day)
                .into_iter()
                .filter(|i| i.all_day)
                .collect()
        })
        .collect();
    let rows = all_day
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(0)
        .min(ALL_DAY_ROWS);
    let top_height = rows as f32 * ALL_DAY_ROW + if rows > 0 { 4.0 } else { 0.0 };
    let mut head = div()
        .class("fs-calendar-head")
        .flex_row()
        .w(width)
        .h(HEAD_HEIGHT);
    head = head.child(div().w(GUTTER));
    let mut top = div()
        .class("fs-calendar-all-day")
        .flex_row()
        .w(width)
        .h(top_height);
    if rows > 0 {
        top = top.child(
            div()
                .class("fs-calendar-gutter")
                .w(GUTTER)
                .child(text(&drawing.names.all_day).no_wrap().no_cursor()),
        );
    } else {
        top = top.child(div().w(GUTTER));
    }
    let mut body = div()
        .flex_row()
        .w(width - SCROLLBAR_ROOM)
        .h(y_of(MINUTES_PER_DAY))
        .flex_shrink_0()
        .child(gutter());
    for (day, tops) in days.iter().zip(&all_day) {
        head = head.child(day_head(drawing, *day, column));
        top = top.child(all_day_cell(drawing, tops, *day, column, top_height));
        let timed: Vec<&CalendarItem> = items_on(items, *day)
            .into_iter()
            .filter(|i| !i.all_day)
            .collect();
        body = body.child(day_column(drawing, &timed, *day, column, axis.now));
    }
    let view = (height - HEAD_HEIGHT - top_height).max(0.0);
    let body = Scroll::with_physics(Arc::clone(&axis.position.physics))
        .bind(&axis.position.scroll)
        .w(width)
        .h(view)
        .content(body);
    div()
        .class("fs-calendar")
        .flex_col()
        .w(width)
        .h(height)
        .child(head)
        .child(top)
        .child(body)
}

fn day_head(drawing: &Drawing, day: Day, width: f32) -> Div {
    let names = &drawing.names;
    let number = div()
        .class("fs-calendar-number")
        .w(24.0)
        .flex_row()
        .items_center()
        .justify_center()
        .child(text(day.day().to_string()).no_wrap().no_cursor());
    let head = div()
        .class("fs-calendar-head-cell")
        .flex_row()
        .items_center()
        .w(width)
        .h(HEAD_HEIGHT)
        .child(
            text(&names.weekdays[usize::from(day.weekday())])
                .no_wrap()
                .no_cursor(),
        )
        .child(number);
    let head = drawing.day_classes(head, day);
    let head = drawing.named(head, &format!("head-{day}"));
    drawing.reports(head, Picked::Day(day))
}

fn all_day_cell(
    drawing: &Drawing,
    items: &[&CalendarItem],
    day: Day,
    width: f32,
    height: f32,
) -> Div {
    let chip_width = (width - 2.0 * ITEM_GAP).max(0.0);
    let mut cell = div().relative().w(width).h(height);
    let shown = if items.len() > ALL_DAY_ROWS {
        ALL_DAY_ROWS - 1
    } else {
        items.len()
    };
    let mut top = 2.0;
    for item in items.iter().take(shown) {
        cell = cell.child(
            drawing
                .chip(item, day, &item.title, chip_width)
                .absolute()
                .left(ITEM_GAP)
                .top(top)
                .w(chip_width)
                .h(ALL_DAY_ROW - ITEM_GAP),
        );
        top += ALL_DAY_ROW;
    }
    if items.len() > shown {
        cell = cell.child(
            drawing
                .more(items.len() - shown, day)
                .absolute()
                .left(ITEM_GAP)
                .top(top)
                .w(chip_width)
                .h(ALL_DAY_ROW - ITEM_GAP),
        );
    }
    cell
}

fn gutter() -> Div {
    let mut gutter = div()
        .w(GUTTER)
        .h(y_of(MINUTES_PER_DAY))
        .flex_col()
        .flex_shrink_0();
    for hour in 0..24u16 {
        let label = if hour == 0 {
            String::new()
        } else {
            Moment::at(Day::from_number(0), hour, 0).clock()
        };
        gutter = gutter.child(
            div()
                .class("fs-calendar-gutter")
                .w(GUTTER)
                .h(HOUR_HEIGHT)
                .child(text(label).no_wrap().no_cursor()),
        );
    }
    gutter
}
