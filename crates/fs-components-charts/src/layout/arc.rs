//! Arcs: a pie or donut, angles proportional to the values, from 12 o'clock, clockwise.
//!
//! Pattern: none (the layout of one mark; `build` picks it by `match` on the mark kind, there is no trait). Values of one series are added up; the series are placed
//! in the natural order of their values (ascending, the first one starts at 12 o'clock and the
//! pile runs clockwise), whatever `color.sort` says: that only moves colours and legend (checked
//! against Vega-Lite 6.5.0, `tests/golden/g02`, `g04*`). A total of zero draws no
//! sector. A thin gap in the surface colour separates sectors so they also differ without colour.

use super::ctx::Ctx;
use super::stack::{Entry, stack};
use crate::scene::{Item, Paint, Scene};
use crate::spec::{Channel, StackMode};

const TAU: f64 = std::f64::consts::TAU;

/// Draw the sectors into `out`; `plot` is `[x, y, w, h]`.
pub fn draw(c: &Ctx<'_>, plot: [f32; 4], out: &mut Scene) {
    let mut entries = Vec::new();
    let (mut at, mut series) = (Vec::new(), Vec::new());
    for (i, d) in c.data.data.iter().enumerate() {
        if let Some(value) = d.cells[Channel::Theta.index()].num() {
            entries.push(Entry::new(0, c.rank_of(d), value));
            at.push(i);
            series.push(c.series_of(d));
        }
    }
    let spans = stack(&entries, StackMode::Normalize, true);
    let outer = (plot[2].min(plot[3]) / 2.0).max(0.0);
    let inner = (c.spec.mark.inner_radius as f32).min(outer);
    let centre = [plot[0] + plot[2] / 2.0, plot[1] + plot[3] / 2.0];
    // a partial pie would mislead: too many sectors are all left out, and the note says so
    if spans.len() > c.max_marks {
        out.omitted += spans.len();
        return;
    }
    for (span, (&i, &s)) in spans.iter().zip(at.iter().zip(&series)) {
        let (start, end) = (span[0] * TAU, span[1] * TAU);
        if end <= start {
            continue;
        }
        out.items.push(Item::Sector {
            centre,
            inner,
            outer,
            start: start as f32,
            end: end as f32,
            paint: Paint::Series(s),
        });
        let mid = f64::midpoint(start, end) as f32;
        let r = f32::midpoint(inner, outer);
        let p = [centre[0] + r * mid.sin(), centre[1] - r * mid.cos()];
        out.hits
            .push(c.hit(&c.data.data[i], [p[0] - 4.0, p[1] - 4.0, 8.0, 8.0]));
    }
}
