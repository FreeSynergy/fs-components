//! The day names row and the all-day row of the timed grid.
//!
//! Pattern: View function — measures from [`Geo`], state in, elements out.

use blinc_layout::div::{Div, div};
use fs_components_fields::Look;

use super::parts::{at, chip, day_number};
use super::timed::{Geo, LABEL_W};
use crate::day::Day;
use crate::handle::CalendarGrid;
use crate::layout::Rect;
use crate::model::{GridItem, Slot};
use crate::state::GridState;

/// The weekday and number of each shown day.
pub fn header(grid: &CalendarGrid, state: &GridState, days: &[Day], geo: &Geo) -> Div {
    let (look, s, pal) = (grid.look, grid.look.style, grid.look.palette());
    let mut row = div().relative().w(geo.width).h(geo.head);
    for (i, day) in days.iter().enumerate() {
        let name = grid.say.say(&format!("calendar.weekday.{}", day.weekday()));
        let (_, _, number) = day.ymd();
        let r = Rect {
            x: LABEL_W + i as f32 * geo.col,
            y: 0.0,
            w: geo.col,
            h: geo.head,
        };
        row = row.child(
            at(r)
                .flex_row()
                .items_center()
                .gap_px(s.spacing.xs)
                .padding_x_px(s.spacing.xs)
                .child(look.text(name, s.text.caption, pal.text_weak))
                .child(day_number(&look, number, state.today == Some(*day), false)),
        );
    }
    row
}

/// The all-day row: one click target per day, the bars of all-day and multi-day items above.
pub fn allday(
    grid: &CalendarGrid,
    state: &GridState,
    days: &[Day],
    items: &[GridItem],
    geo: &Geo,
) -> Div {
    let (look, pal) = (grid.look, grid.look.palette());
    let mut row = div().relative().w(geo.width).h(geo.allday).id("cal-allday");
    for (i, day) in days.iter().enumerate() {
        let slot = Slot {
            day: *day,
            minute: None,
        };
        let g = grid.clone();
        let r = Rect {
            x: LABEL_W + i as f32 * geo.col,
            y: 0.0,
            w: geo.col,
            h: geo.allday,
        };
        let mut cell = at(r)
            .id(format!("cal-allday-{i}"))
            .border(1.0, Look::color(pal.border))
            .on_click(move |_| g.select_slot(slot));
        if state.slot == Some(slot) {
            cell = cell.bg(Look::color(pal.selected));
        }
        row = row.child(cell);
    }
    for b in &geo.bars {
        let it = &items[b.index];
        let r = Rect {
            x: LABEL_W + b.first as f32 * geo.col + 2.0,
            y: 2.0 + b.lane as f32 * (geo.bar + 2.0),
            w: (b.last - b.first + 1) as f32 * geo.col - 4.0,
            h: geo.bar,
        };
        let (g, key) = (grid.clone(), it.key.clone());
        let selected = state.item.as_deref() == Some(it.key.as_str());
        row = row.child(
            chip(&look, r, it.tone, &it.title, selected)
                .id(format!("cal-item-{}", it.key))
                .on_click(move |_| g.select_item(&key)),
        );
    }
    row
}
