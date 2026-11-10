//! What of a tile is drawn: inside the view, big enough to see, and within the budget.
//!
//! Pattern: Strategy by cost — each shape has an estimated number of vertices; shapes are taken in
//! the order given (nearest tiles first) until the budget of their kind is spent, and what is not
//! taken is counted, never dropped silently. The numbers are measured on Blinc 0.6 (`DESIGN.md`).

use std::sync::Arc;

use super::model::TilePoint;
use super::scene::{Placed, bounds, touches};

/// Estimated vertices Blinc makes per point of a filled ring.
pub(super) const FILL_COST: usize = 2;
/// Estimated vertices Blinc makes per point of a stroke (butt caps, mitre joins).
pub(super) const STROKE_COST: usize = 4;
/// Vertices the areas of one picture may cost.
pub const AREA_BUDGET: usize = 30_000;
/// Vertices the lines of one picture may cost.
pub const LINE_BUDGET: usize = 18_000;
/// Dots one picture may draw (they are not paths; Blinc's limit is 20,000 shapes a frame).
pub const DOT_BUDGET: usize = 8_000;
/// A shape whose box is smaller than this many pixels on both sides is not drawn.
pub(super) const TOO_SMALL: f32 = 0.75;

/// What is left of the budgets.
pub(super) struct Budget {
    pub(super) areas: usize,
    pub(super) lines: usize,
    pub(super) dots: usize,
    pub(super) omitted: usize,
}

impl Budget {
    fn take(left: &mut usize, cost: usize, omitted: &mut usize) -> bool {
        if *left >= cost {
            *left -= cost;
            true
        } else {
            *omitted += 1;
            false
        }
    }
}

pub(super) fn choose(mut tile: Placed, (w, h): (f32, f32), budget: &mut Budget) -> Placed {
    let geometry = Arc::clone(&tile.geometry);
    let seen = |points: &[TilePoint], min_size: f32| {
        bounds(points.iter().map(|p| tile.to_screen(*p))).is_some_and(|(lo, hi)| {
            touches(lo, hi, w, h) && (hi[0] - lo[0] >= min_size || hi[1] - lo[1] >= min_size)
        })
    };
    let mut chosen = (Vec::new(), Vec::new(), Vec::new());
    for (i, area) in geometry.areas.iter().enumerate() {
        let Some(outer) = area.rings.first().filter(|r| seen(r, TOO_SMALL)) else {
            continue;
        };
        let points: usize = area.rings.iter().map(Vec::len).sum();
        let stroked = if area.outline.is_some() {
            STROKE_COST
        } else {
            0
        };
        let cost = points * (FILL_COST + stroked);
        if outer.len() >= 3 && Budget::take(&mut budget.areas, cost, &mut budget.omitted) {
            chosen.0.push(i);
        }
    }
    for (i, line) in geometry.lines.iter().enumerate() {
        if line.points.len() >= 2
            && seen(&line.points, TOO_SMALL)
            && Budget::take(
                &mut budget.lines,
                line.points.len() * STROKE_COST,
                &mut budget.omitted,
            )
        {
            chosen.1.push(i);
        }
    }
    for (i, dot) in geometry.dots.iter().enumerate() {
        let [x, y] = tile.to_screen(dot.at);
        if touches([x, y], [x, y], w, h) && Budget::take(&mut budget.dots, 1, &mut budget.omitted) {
            chosen.2.push(i);
        }
    }
    (tile.areas, tile.lines, tile.dots) = chosen;
    tile
}
