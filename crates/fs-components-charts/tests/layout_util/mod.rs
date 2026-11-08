//! Helpers shared by the layout tests.

#![allow(dead_code)] // each test file uses a part of these helpers

use fs_components_charts::layout::{Fixed, Metrics};
use fs_components_charts::scene::{Item, Scene};
use fs_components_charts::{Chart, Labels};

pub fn scene(json: &str, w: f32, h: f32) -> Scene {
    Chart::from_json(json)
        .unwrap_or_else(|e| panic!("{e}"))
        .scene(w, h, &Labels::default(), &Fixed, &Metrics::default())
}

pub fn doc(mark: &str, rows: &str, enc: &str) -> String {
    format!(r#"{{"data":{{"values":{rows}}},"mark":"{mark}","encoding":{{{enc}}}}}"#)
}

pub const CAT: &str =
    r#""x":{"field":"k","type":"nominal"},"y":{"field":"v","type":"quantitative"}"#;
pub const TIME: &str =
    r#""x":{"field":"d","type":"temporal"},"y":{"field":"v","type":"quantitative"}"#;

pub fn texts(s: &Scene) -> Vec<String> {
    s.items
        .iter()
        .filter_map(|i| {
            if let Item::Text { text, .. } = i {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect()
}

pub fn rects(s: &Scene) -> Vec<[f32; 4]> {
    s.items
        .iter()
        .filter_map(|i| {
            if let Item::Rect { x, y, w, h, .. } = i {
                Some([*x, *y, *w, *h])
            } else {
                None
            }
        })
        .collect()
}

pub fn finite(s: &Scene) -> bool {
    let ok = |v: &[f32]| v.iter().all(|x| x.is_finite());
    s.items.iter().all(|i| match i {
        Item::Rect { x, y, w, h, .. } => ok(&[*x, *y, *w, *h]),
        Item::Line { from, to, .. } => ok(from) && ok(to),
        Item::Path { points, .. } | Item::Polygon { points, .. } => points.iter().all(|p| ok(p)),
        Item::Marker { at, .. } | Item::Text { at, .. } => ok(at),
        Item::Sector {
            centre, start, end, ..
        } => ok(centre) && ok(&[*start, *end]),
        _ => true,
    })
}
