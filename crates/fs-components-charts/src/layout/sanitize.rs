//! The last guard: nothing with a coordinate that is not a usable number reaches the drawing.
//!
//! Pattern: none. A NaN or infinite coordinate (from a domain too narrow to divide by, an
//! overflow) would panic Blinc's tessellator or draw garbage; the shape is left out and counted in
//! `Scene::omitted`, so the chart says so under the plot instead of staying blank.

use crate::scene::{Item, Scene};

const LIMIT: f32 = 1e7;

fn ok(v: &[f32]) -> bool {
    v.iter().all(|x| x.is_finite() && x.abs() <= LIMIT)
}

fn usable(i: &Item) -> bool {
    match i {
        Item::Rect { x, y, w, h, .. } => ok(&[*x, *y, *w, *h]),
        Item::Line { from, to, .. } => ok(from) && ok(to),
        Item::Path { points, .. } | Item::Polygon { points, .. } => points.iter().all(|p| ok(p)),
        Item::Marker { at, radius, .. } => ok(at) && ok(&[*radius]),
        Item::Sector {
            centre,
            inner,
            outer,
            start,
            end,
            ..
        } => ok(centre) && ok(&[*inner, *outer, *start, *end]),
        Item::Text { at, .. } => ok(at),
    }
}

/// Remove unusable shapes; count them as omitted.
pub fn clean(scene: &mut Scene) {
    let before = scene.items.len();
    let from = scene.marks_from.min(before);
    let mut kept = 0;
    let mut new_from = 0;
    for (n, item) in std::mem::take(&mut scene.items).into_iter().enumerate() {
        if n == from {
            new_from = kept;
        }
        if usable(&item) {
            scene.items.push(item);
            kept += 1;
        }
    }
    if from >= before {
        new_from = kept;
    }
    scene.omitted += before - kept;
    scene.marks_from = new_from;
    scene.hits.retain(|h| ok(&h.area));
}
