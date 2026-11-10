//! The whole timeline: toolbar, then the axis, rows, arrows and today line in one scroll box.
//!
//! Pattern: View function — the source is read once per build; sizes come from `layout`.
//! One scroll box (both directions) holds everything: the axis scrolls away with the rows
//! (sticky header and labels are listed as Unsupported in DESIGN.md).

use blinc_layout::div::{Div, div};
use blinc_layout::widgets::scroll::scroll;

use super::{arrows, axis, rows, toolbar};
use crate::handle::Timeline;
use crate::layout::{Rect, Rows, Scale, item_rect, range};
use crate::model::Day;
use crate::route::route;

/// The timeline filling `width` x `height`.
pub fn timeline(t: &Timeline, width: f32, height: f32) -> Div {
    let (s, st) = (t.look.style, t.snapshot());
    let items = t.source.items();
    let (first, last) = range(&items, 7, st.today.unwrap_or(Day(0)));
    let scale = Scale {
        origin: first,
        px: st.zoom.px_per_day(),
    };
    let g = Rows {
        row: s.heights.row_dense + s.spacing.xs,
        bar: s.heights.row_dense - s.spacing.xs,
    };
    let head = 2.0 * axis::row_height(t);
    let (total_w, rows_h) = (
        scale.x_of(last.add_days(1)).max(width),
        items.len() as f32 * g.row,
    );
    let mut body = div().relative().w(total_w).h(head + rows_h).id("tl-body");
    for e in axis::axis(t, st.zoom, (first, last), &scale, head + rows_h) {
        body = body.child(e);
    }
    let rect_of = |key: &str| {
        items
            .iter()
            .position(|i| i.key == key)
            .map(|r| item_rect(&items[r], r, &scale, g))
    };
    let routes: Vec<_> = t
        .source
        .dependencies()
        .iter()
        .filter_map(|d| Some(route(rect_of(&d.from)?, rect_of(&d.to)?, g.row)))
        .collect();
    body = body.child(arrows::arrows(
        &t.look,
        Rect {
            x: 0.0,
            y: head,
            w: total_w,
            h: rows_h,
        },
        routes,
    ));
    for (row, it) in items.iter().enumerate() {
        body = body.child(rows::item(
            t,
            it,
            row,
            &scale,
            g,
            head,
            st.selected.as_deref() == Some(it.key.as_str()),
        ));
    }
    if let Some(today) = st.today.filter(|d| *d >= first && *d <= last) {
        body = body.child(rows::today_line(
            t,
            scale.x_of(today) + scale.px / 2.0 - 1.0,
            head + rows_h,
        ));
    }
    let bar = s.heights.control + s.spacing.s;
    div()
        .flex_col()
        .w(width)
        .h(height)
        .child(toolbar::toolbar(t))
        .child(
            scroll()
                .both_directions()
                .w(width)
                .h((height - bar).max(0.0))
                .content(body),
        )
}
