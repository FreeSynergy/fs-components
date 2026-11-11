//! Where labels and markers go on the screen.
//!
//! Pattern: Greedy placement — labels are taken nearest tile first and one that would overlap a label
//! already placed is left out; a marker outside the view is left out.

use super::model::Marker;
use super::position::MapPosition;
use super::scene::{CHAR_WIDTH, LIMIT, MOST_LABELS, Placed, PlacedLabel, PlacedMarker, touches};

#[allow(clippy::cast_precision_loss)]
pub(super) fn place_labels(tiles: &[Placed], w: f32, h: f32) -> Vec<PlacedLabel> {
    let mut taken: Vec<[f32; 4]> = Vec::new();
    let mut labels = Vec::new();
    for (t, tile) in tiles.iter().enumerate() {
        for (index, label) in tile.geometry.labels.iter().enumerate() {
            if labels.len() >= MOST_LABELS || label.text.is_empty() {
                continue;
            }
            let [x, y] = tile.to_screen(label.at);
            let half = label.text.chars().count() as f32 * label.size * CHAR_WIDTH / 2.0;
            let rect = [
                x - half,
                y - label.size / 2.0,
                x + half,
                y + label.size / 2.0,
            ];
            let overlaps =
                |o: &[f32; 4]| rect[0] < o[2] && o[0] < rect[2] && rect[1] < o[3] && o[1] < rect[3];
            if !touches([rect[0], rect[1]], [rect[2], rect[3]], w, h) || taken.iter().any(overlaps)
            {
                continue;
            }
            taken.push(rect);
            labels.push(PlacedLabel {
                text: label.text.clone(),
                x,
                y,
                size: label.size,
                tile: t,
                index,
            });
        }
    }
    labels
}

pub(super) fn place_markers(
    position: MapPosition,
    (w, h): (f64, f64),
    markers: &[Marker],
) -> Vec<PlacedMarker> {
    markers
        .iter()
        .filter(|m| m.lon.is_finite() && m.lat.is_finite())
        .filter_map(|m| {
            let (x, y) = position.to_screen(m.lon, m.lat, w, h);
            #[allow(clippy::cast_possible_truncation)]
            let (x, y) = (x.clamp(-LIMIT, LIMIT) as f32, y.clamp(-LIMIT, LIMIT) as f32);
            #[allow(clippy::cast_possible_truncation)]
            let inside = touches(
                [x - 20.0, y - 40.0],
                [x + 20.0, y + 20.0],
                w as f32,
                h as f32,
            );
            inside.then(|| PlacedMarker {
                marker: m.clone(),
                x,
                y,
            })
        })
        .collect()
}
