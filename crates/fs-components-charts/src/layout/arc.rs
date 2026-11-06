//! Arcs: a pie or donut, angles proportional to the values, from 12 o'clock, clockwise.
//!
//! Pattern: Strategy (one mark layout). Values of one series are added up; the series are placed
//! in Vega-Lite's stack order (series 0 last, so it ends at 12 o'clock). A total of zero draws no
//! sector. A thin gap in the surface colour separates sectors so they also differ without colour.

use super::ctx::Ctx;
use super::stack::{Entry, stack};
use crate::scene::{Item, Paint, Scene};
use crate::spec::{Channel, StackMode};

const TAU: f64 = std::f64::consts::TAU;

/// Draw the sectors into `out`; `plot` is `[x, y, w, h]`.
pub fn draw(c: &Ctx<'_>, plot: [f32; 4], out: &mut Scene) {
    let mut entries = Vec::new();
    let mut at = Vec::new();
    for (i, d) in c.data.data.iter().enumerate() {
        if let Some(value) = d.cells[Channel::Theta.index()].num() {
            entries.push(Entry {
                group: 0,
                series: c.series_of(d),
                value,
            });
            at.push(i);
        }
    }
    let spans = stack(&entries, StackMode::Normalize);
    let outer = (plot[2].min(plot[3]) / 2.0).max(0.0);
    let inner = (c.spec.mark.inner_radius as f32).min(outer);
    let centre = [plot[0] + plot[2] / 2.0, plot[1] + plot[3] / 2.0];
    out.omitted += spans.len().saturating_sub(c.max_marks);
    for (span, (&i, e)) in spans.iter().zip(at.iter().zip(&entries)).take(c.max_marks) {
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
            paint: Paint::Series(e.series),
        });
        let mid = f64::midpoint(start, end) as f32;
        let r = f32::midpoint(inner, outer);
        let p = [centre[0] + r * mid.sin(), centre[1] - r * mid.cos()];
        out.hits
            .push(c.hit(&c.data.data[i], [p[0] - 4.0, p[1] - 4.0, 8.0, 8.0]));
    }
}
