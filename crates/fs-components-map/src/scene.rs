//! The plan of one picture of the map: what is where on the screen, before anything is drawn.
//!
//! Pattern: Builder of a value — from a position, a size, the tiles the caller supplied and the
//! markers, it works out screen positions, chooses what is drawn (what lies outside the view or is
//! smaller than a pixel is not), keeps to a budget, and places labels so they do not overlap. Drawing
//! only walks the result, so this part is tested without graphics and is what is timed.
//!
//! The budget is Blinc's: it draws about 65,000 path vertices per frame, the whole window together,
//! and silently drops the rest (measured; see `DESIGN.md`). So the plan counts an estimate of the
//! vertices of what it chooses, nearest tiles first, and reports what it left out.

use std::sync::Arc;

use super::choose::{AREA_BUDGET, Budget, DOT_BUDGET, LINE_BUDGET, choose};
use super::mercator::finite_or;
use super::model::{Marker, TileGeometry, TilePoint};
use super::place::{place_labels, place_markers};
use super::position::MapPosition;
use super::tiles::NeededTile;

/// Screen coordinates are kept within this, so an extreme overzoom cannot hand the renderer a
/// number it cannot hold.
pub(super) const LIMIT: f64 = 1.0e6;
/// The most labels one picture carries.
pub const MOST_LABELS: usize = 300;
/// A character is about this wide, in units of the text size.
pub const CHAR_WIDTH: f32 = 0.56;
/// A tile with its place on the screen and what of it is drawn.
#[derive(Clone, Debug)]
pub struct Placed {
    /// What is drawn.
    pub geometry: Arc<TileGeometry>,
    /// The tile's screen rectangle.
    pub at: NeededTile,
    /// Indexes of the areas drawn.
    pub areas: Vec<usize>,
    /// Indexes of the lines drawn.
    pub lines: Vec<usize>,
    /// Indexes of the dots drawn.
    pub dots: Vec<usize>,
}

impl Placed {
    /// A point of the tile on the screen.
    #[allow(clippy::cast_possible_truncation)]
    pub fn to_screen(&self, p: TilePoint) -> [f32; 2] {
        let scale = self.at.size / f64::from(self.geometry.extent);
        let x = self.at.left + f64::from(p[0]) * scale;
        let y = self.at.top + f64::from(p[1]) * scale;
        [
            finite_or(x, 0.0).clamp(-LIMIT, LIMIT) as f32,
            finite_or(y, 0.0).clamp(-LIMIT, LIMIT) as f32,
        ]
    }
}

/// A marker with its place on the screen.
#[derive(Clone, Debug, PartialEq)]
pub struct PlacedMarker {
    /// The marker.
    pub marker: Marker,
    /// Pixel position.
    pub x: f32,
    /// Pixel position.
    pub y: f32,
}

/// A label that found room.
#[derive(Clone, Debug, PartialEq)]
pub struct PlacedLabel {
    /// The text.
    pub text: String,
    /// Where its middle is.
    pub x: f32,
    /// Where its middle is.
    pub y: f32,
    /// Size in pixels.
    pub size: f32,
    /// Index of the tile in [`Scene::tiles`].
    pub tile: usize,
    /// Index in the tile's labels.
    pub index: usize,
}

/// What one picture holds.
#[derive(Clone, Debug)]
pub struct Scene {
    /// Width of the view in pixels.
    pub width: f32,
    /// Height of the view in pixels.
    pub height: f32,
    /// The tiles, in drawing order (the farthest from the centre first).
    pub tiles: Vec<Placed>,
    /// Labels that found room.
    pub labels: Vec<PlacedLabel>,
    /// The markers inside the view.
    pub markers: Vec<PlacedMarker>,
    /// Shapes in view that the budget did not allow: they are not drawn.
    pub omitted: usize,
}

/// Do points with this bounding box (in screen pixels) touch the view?
pub fn touches(min: [f32; 2], max: [f32; 2], width: f32, height: f32) -> bool {
    max[0] >= 0.0 && max[1] >= 0.0 && min[0] <= width && min[1] <= height
}

/// The box around screen points; `None` for no points.
pub fn bounds(points: impl Iterator<Item = [f32; 2]>) -> Option<([f32; 2], [f32; 2])> {
    points.fold(None, |acc, p| match acc {
        None => Some((p, p)),
        Some((lo, hi)) => Some((
            [lo[0].min(p[0]), lo[1].min(p[1])],
            [hi[0].max(p[0]), hi[1].max(p[1])],
        )),
    })
}

impl Scene {
    /// The plan for a view `width` x `height` at `position`. `tiles` come with their rectangles,
    /// nearest the centre first, as `needed_tiles` gives them; a marker outside is left out.
    pub fn new(
        position: MapPosition,
        width: f32,
        height: f32,
        tiles: Vec<(NeededTile, Arc<TileGeometry>)>,
        markers: &[Marker],
    ) -> Self {
        let width = finite_or(f64::from(width), 0.0).max(0.0);
        let height = finite_or(f64::from(height), 0.0).max(0.0);
        #[allow(clippy::cast_possible_truncation)]
        let (w, h) = (width as f32, height as f32);
        let mut budget = Budget {
            areas: AREA_BUDGET,
            lines: LINE_BUDGET,
            dots: DOT_BUDGET,
            omitted: 0,
        };
        let mut placed: Vec<Placed> = tiles
            .into_iter()
            .map(|(at, geometry)| {
                choose(
                    Placed {
                        geometry,
                        at,
                        areas: Vec::new(),
                        lines: Vec::new(),
                        dots: Vec::new(),
                    },
                    (w, h),
                    &mut budget,
                )
            })
            .collect();
        let labels = place_labels(&placed, w, h);
        placed.reverse();
        let count = placed.len();
        let labels = labels
            .into_iter()
            .map(|mut l| {
                l.tile = count - 1 - l.tile;
                l
            })
            .collect();
        Self {
            width: w,
            height: h,
            tiles: placed,
            labels,
            markers: place_markers(position, (width, height), markers),
            omitted: budget.omitted,
        }
    }

    /// How many shapes of tiles are drawn.
    pub fn drawn(&self) -> usize {
        self.tiles
            .iter()
            .map(|t| t.areas.len() + t.lines.len() + t.dots.len())
            .sum()
    }
}
