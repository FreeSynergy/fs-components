//! Paths for symbols and sectors.
//!
//! Pattern: none — points in, `Path` out; no state.

use blinc_core::Path;

use crate::scene::Shape;

/// Corners of a symbol of the given area-equivalent radius, as a closed polygon. `Cross` and
/// `Circle` are drawn by the caller and give no polygon.
pub fn polygon(shape: Shape, c: [f32; 2], r: f32) -> Option<Vec<[f32; 2]>> {
    let [x, y] = c;
    Some(match shape {
        Shape::Square => {
            let h = r * 0.886; // same area as the circle: side = r * sqrt(pi)
            vec![
                [x - h, y - h],
                [x + h, y - h],
                [x + h, y + h],
                [x - h, y + h],
            ]
        }
        Shape::Diamond => {
            let h = r * 1.253;
            vec![[x, y - h], [x + h, y], [x, y + h], [x - h, y]]
        }
        Shape::TriangleUp => {
            let (h, w) = (r * 1.4, r * 1.45);
            vec![[x, y - h], [x + w, y + h * 0.7], [x - w, y + h * 0.7]]
        }
        Shape::TriangleDown => {
            let (h, w) = (r * 1.4, r * 1.45);
            vec![[x, y + h], [x + w, y - h * 0.7], [x - w, y - h * 0.7]]
        }
        _ => return None,
    })
}

/// A closed path through the points.
pub fn closed(points: &[[f32; 2]]) -> Path {
    let mut it = points.iter();
    let Some(first) = it.next() else {
        return Path::new();
    };
    let mut p = Path::new().move_to(first[0], first[1]);
    for q in it {
        p = p.line_to(q[0], q[1]);
    }
    p.close()
}

/// An open path through the points.
pub fn open(points: &[[f32; 2]]) -> Path {
    let mut it = points.iter();
    let Some(first) = it.next() else {
        return Path::new();
    };
    let mut p = Path::new().move_to(first[0], first[1]);
    for q in it {
        p = p.line_to(q[0], q[1]);
    }
    p
}

/// A ring sector from `start` to `end` (radians, 0 at 12 o'clock, clockwise).
pub fn sector(c: [f32; 2], inner: f32, outer: f32, start: f32, end: f32) -> Path {
    let steps = (((end - start).abs() / 0.05).ceil() as usize).max(2);
    let at = |r: f32, a: f32| [c[0] + r * a.sin(), c[1] - r * a.cos()];
    let mut pts: Vec<[f32; 2]> = Vec::with_capacity(2 * steps + 2);
    for i in 0..=steps {
        pts.push(at(outer, start + (end - start) * i as f32 / steps as f32));
    }
    if inner > 0.0 {
        for i in (0..=steps).rev() {
            pts.push(at(inner, start + (end - start) * i as f32 / steps as f32));
        }
    } else {
        pts.push(c);
    }
    closed(&pts)
}
