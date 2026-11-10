//! The axis: two rows of labels and thin lines down the rows.
//!
//! Pattern: View function — ticks from `layout`; month names come from `Say`
//! (`timeline.month.1..12`), the day numbers and years are plain numbers.

use blinc_layout::div::{Div, div};
use fs_components_fields::Look;

use super::parts::at;
use crate::handle::Timeline;
use crate::layout::{Rect, Scale, Tick, TickLabel, month_marks, ticks};
use crate::model::{Day, Zoom};

/// Height of one of the two label rows.
pub fn row_height(t: &Timeline) -> f32 {
    t.look.style.heights.row_dense
}

fn label(t: &Timeline, tick: &Tick) -> String {
    match tick.label {
        TickLabel::Day(n) => n.to_string(),
        TickLabel::Month(m, y) => format!("{} {y}", t.say.say(&format!("timeline.month.{m}"))),
    }
}

/// The header (labels) and the lines below it down to `total_h`.
pub fn axis(
    t: &Timeline,
    zoom: Zoom,
    (first, last): (Day, Day),
    scale: &Scale,
    total_h: f32,
) -> Vec<Div> {
    let (look, s, pal) = (t.look, t.look.style, t.look.palette());
    let h = row_height(t);
    let mut out = Vec::new();
    let mut put = |tick: &Tick, y: f32, line: bool| {
        let r = Rect {
            x: scale.x_of(tick.day),
            y,
            w: scale.px * f32::from(tick.days),
            h,
        };
        out.push(
            at(r)
                .padding_x_px(s.spacing.xs)
                .pointer_events_none()
                .child(look.text(label(t, tick), s.text.caption, pal.text_weak)),
        );
        if line {
            out.push(
                at(Rect {
                    x: r.x,
                    y: 0.0,
                    w: 1.0,
                    h: total_h,
                })
                .bg(Look::color(pal.border))
                .pointer_events_none(),
            );
        }
    };
    if zoom == Zoom::Month {
        for k in &ticks(zoom, first, last) {
            put(k, 0.0, true);
        }
    } else {
        for k in &month_marks(first, last) {
            put(k, 0.0, false);
        }
        for k in &ticks(zoom, first, last) {
            put(k, h, true);
        }
    }
    out.push(
        div()
            .absolute()
            .left(0.0)
            .top(2.0 * h - 1.0)
            .w(scale.x_of(last.add_days(1)))
            .h(1.0)
            .bg(Look::color(pal.border)),
    );
    out
}
