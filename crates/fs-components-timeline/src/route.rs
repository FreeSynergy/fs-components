//! The path of a dependency arrow from the end of one item to the start of the next.
//!
//! Pattern: none — a pure function from two rectangles to points; the view draws the points on a
//! canvas, tests check them without one.

use crate::layout::Rect;

/// A point in pixels.
pub type Point = (f32, f32);

/// Distance of the first and last stretch from the items.
pub const GAP: f32 = 8.0;
/// Length of the arrow head.
pub const HEAD: f32 = 6.0;

/// An arrow: the polyline and the triangle of its head.
#[derive(Clone, Debug, PartialEq)]
pub struct Arrow {
    /// The line from the end of the first item to the head.
    pub line: Vec<Point>,
    /// The head, tip first.
    pub head: [Point; 3],
}

/// The arrow from `from` (leaves at its right edge) to `to` (enters at its left edge). When `to`
/// starts before `from` ends, the line goes round through the gap between the rows.
pub fn route(from: Rect, to: Rect, row: f32) -> Arrow {
    let (xe, y1) = (from.x + from.w, from.y + from.h / 2.0);
    let (xs, y2) = (to.x, to.y + to.h / 2.0);
    let line = if xs >= xe + 2.0 * GAP {
        if (y1 - y2).abs() < 0.5 {
            vec![(xe, y1), (xs, y2)]
        } else {
            vec![(xe, y1), (xe + GAP, y1), (xe + GAP, y2), (xs, y2)]
        }
    } else {
        let mid = if y2 < y1 {
            y1 - row / 2.0
        } else {
            y1 + row / 2.0
        };
        vec![
            (xe, y1),
            (xe + GAP, y1),
            (xe + GAP, mid),
            (xs - GAP, mid),
            (xs - GAP, y2),
            (xs, y2),
        ]
    };
    Arrow {
        line,
        head: [
            (xs, y2),
            (xs - HEAD, y2 - HEAD / 2.0),
            (xs - HEAD, y2 + HEAD / 2.0),
        ],
    }
}
