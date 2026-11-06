//! Points: one symbol per datum.
//!
//! Pattern: Strategy (one mark layout). Vega-Lite's point is an open circle of area 30 with a 2 px
//! line, at 0.7 opacity for plots that are not aggregated. With several series the symbol also
//! changes shape, so series differ without colour too (an addition for colour-blind readers).

use super::axis::Axis;
use super::ctx::{Ctx, shape_of};
use crate::scene::{Item, Paint, Scene, Shape};
use crate::spec::Channel;

const RADIUS: f32 = 3.09; // sqrt(30 / pi): a circle of area 30

/// Draw all points.
pub fn draw(c: &Ctx<'_>, x: &Axis, y: &Axis, plot: [f32; 4], out: &mut Scene) {
    let aggregated = c
        .spec
        .encoding
        .y
        .as_ref()
        .is_some_and(|d| d.aggregate.is_some())
        || c.spec
            .encoding
            .x
            .as_ref()
            .is_some_and(|d| d.aggregate.is_some());
    let opacity = if aggregated { 1.0 } else { 0.7 };
    for d in &c.data.data {
        let (Some(px), Some(py)) = (
            x.place(&d.cells[Channel::X.index()], c.labels),
            y.place(&d.cells[Channel::Y.index()], c.labels),
        ) else {
            continue;
        };
        let s = c.series_of(d);
        let at = [plot[0] + px, plot[1] + py];
        let shape = if c.multi() {
            shape_of(s)
        } else {
            Shape::Circle
        };
        out.items.push(Item::Marker {
            at,
            shape,
            radius: RADIUS,
            paint: Paint::Series(s),
            filled: false,
            opacity,
        });
        out.hits.push(c.hit(
            d,
            [at[0] - RADIUS, at[1] - RADIUS, 2.0 * RADIUS, 2.0 * RADIUS],
        ));
    }
}
