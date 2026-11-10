//! The scrolling time axis: hour lines and labels, one column per day, items, the chosen slot, now.
//!
//! Pattern: View function — minutes to pixels by `axis`; a click works out its minute from the pointer.

use blinc_layout::div::{Div, div};
use fs_components_fields::Look;

use super::parts::{at, chip};
use super::timed::{Geo, LABEL_W};
use crate::axis::{day_segments, lanes, minute_at, y_of};
use crate::day::Day;
use crate::handle::CalendarGrid;
use crate::layout::Rect;
use crate::model::{GridItem, Slot};
use crate::state::GridState;

/// The whole axis, 24 hours high.
pub fn body(
    grid: &CalendarGrid,
    state: &GridState,
    days: &[Day],
    items: &[GridItem],
    geo: &Geo,
) -> Div {
    let (look, pal) = (grid.look, grid.look.palette());
    let high = 24.0 * geo.hour;
    let mut root = div().relative().w(geo.width).h(high);
    root = hours(grid, root, geo);
    for (i, day) in days.iter().enumerate() {
        let x = LABEL_W + i as f32 * geo.col;
        let (g, d, snap, hour) = (grid.clone(), *day, state.slot_minutes, geo.hour);
        root = root.child(
            at(Rect {
                x,
                y: 0.0,
                w: geo.col,
                h: high,
            })
            .id(format!("cal-col-{i}"))
            .border(1.0, Look::color(pal.border))
            .on_click(move |ctx| {
                g.select_slot(Slot {
                    day: d,
                    minute: Some(minute_at(ctx.local_y, hour, snap)),
                });
            }),
        );
        if let Some(Slot {
            day: sd,
            minute: Some(m),
        }) = state.slot
            && sd == *day
        {
            let r = Rect {
                x,
                y: y_of(m, geo.hour),
                w: geo.col,
                h: y_of(state.slot_minutes, geo.hour),
            };
            root = root.child(
                at(r)
                    .bg(Look::color(pal.selected))
                    .pointer_events_none()
                    .id("cal-slot"),
            );
        }
        for p in lanes(&day_segments(items, *day), 30) {
            let it = &items[p.segment.index];
            let w = geo.col / p.lanes as f32;
            let r = Rect {
                x: x + p.lane as f32 * w + 1.0,
                y: y_of(p.segment.start, geo.hour),
                w: w - 2.0,
                h: y_of(p.segment.end - p.segment.start, geo.hour).max(geo.bar),
            };
            let (g, key) = (grid.clone(), it.key.clone());
            let selected = state.item.as_deref() == Some(it.key.as_str());
            root = root.child(
                chip(&look, r, it.tone, &it.title, selected)
                    .id(format!("cal-item-{}", it.key))
                    .on_click(move |_| g.select_item(&key)),
            );
        }
        if let Some(now) = state.now
            && now.day == *day
        {
            root = root.child(
                at(Rect {
                    x,
                    y: y_of(now.minute, geo.hour),
                    w: geo.col,
                    h: 2.0,
                })
                .bg(Look::color(pal.error))
                .pointer_events_none()
                .id("cal-now"),
            );
        }
    }
    root
}

/// The hour lines and their labels.
fn hours(grid: &CalendarGrid, mut root: Div, geo: &Geo) -> Div {
    let (look, s, pal) = (grid.look, grid.look.style, grid.look.palette());
    for h in 0..24u16 {
        let y = f32::from(h) * geo.hour;
        root = root
            .child(
                at(Rect {
                    x: LABEL_W,
                    y,
                    w: geo.width - LABEL_W,
                    h: 1.0,
                })
                .bg(Look::color(pal.border))
                .pointer_events_none(),
            )
            .child(
                at(Rect {
                    x: 0.0,
                    y,
                    w: LABEL_W,
                    h: geo.hour,
                })
                .padding_x_px(s.spacing.xs)
                .pointer_events_none()
                .child(look.text(
                    format!("{h:02}:00"),
                    s.text.caption,
                    pal.text_weak,
                )),
            );
    }
    root
}
