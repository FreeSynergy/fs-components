//! The view of a map: where its centre is and how far it is zoomed, as a value.
//!
//! Pattern: Value object — `MapPosition` is immutable and always valid: every constructor and every
//! step passes through the same sanitising, so no sequence of pans and zooms, however extreme, leaves
//! it with a latitude beyond the poles, a longitude outside the world, or a number that is not one.
//! Pan and zoom are plain values here; gestures and keys come from the frontend (ADR 0037).

use super::mercator::{
    MAX_ZOOM, MIN_ZOOM, clamp_lat, clamp_zoom, finite_or, project, span_degrees, unproject,
    wrap_lon,
};

/// The most pixels one pan or one zoom-at-point step may move: beyond it the number is a mistake.
const MOST_PIXELS: f64 = 1.0e7;

/// Centre and zoom of a map view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapPosition {
    lon: f64,
    lat: f64,
    zoom: f64,
}

impl Default for MapPosition {
    fn default() -> Self {
        Self::world()
    }
}

impl MapPosition {
    /// A position; a longitude is wrapped into the world, a latitude and a zoom clamped, and
    /// anything that is not a number replaced.
    pub fn new(lon: f64, lat: f64, zoom: f64) -> Self {
        Self {
            lon: wrap_lon(lon),
            lat: clamp_lat(lat),
            zoom: clamp_zoom(zoom),
        }
    }

    /// The whole world at the lowest zoom.
    pub fn world() -> Self {
        Self::new(0.0, 0.0, MIN_ZOOM)
    }

    /// Longitude of the centre, in `[-180, 180)`.
    pub fn lon(&self) -> f64 {
        self.lon
    }

    /// Latitude of the centre.
    pub fn lat(&self) -> f64 {
        self.lat
    }

    /// The zoom: 0 shows the world in one tile, each step doubles the scale.
    pub fn zoom(&self) -> f64 {
        self.zoom
    }

    /// The same centre at another zoom.
    #[must_use]
    pub fn with_zoom(self, zoom: f64) -> Self {
        Self::new(self.lon, self.lat, zoom)
    }

    /// The same zoom around another centre.
    #[must_use]
    pub fn centred_on(self, lon: f64, lat: f64) -> Self {
        Self::new(lon, lat, self.zoom)
    }

    /// The centre moved by `dx` pixels east and `dy` pixels south on the screen (a swipe to the left
    /// is a positive `dx`). The latitude stops at the poles' limit; the longitude wraps.
    #[must_use]
    pub fn panned_by(self, dx: f64, dy: f64) -> Self {
        let dx = finite_or(dx, 0.0).clamp(-MOST_PIXELS, MOST_PIXELS);
        let dy = finite_or(dy, 0.0).clamp(-MOST_PIXELS, MOST_PIXELS);
        let (x, y) = project(self.lon, self.lat, self.zoom);
        let (lon, lat) = unproject(x + dx, y + dy, self.zoom);
        Self::new(lon, lat, self.zoom)
    }

    /// Zoomed by `steps` (negative to zoom out), around the centre; the zoom stays inside its limits.
    #[must_use]
    pub fn zoomed_by(self, steps: f64) -> Self {
        self.with_zoom(self.zoom + finite_or(steps, 0.0))
    }

    /// Zoomed by `steps` so that the point at `(x, y)` pixels of a view `width` x `height` stays under
    /// the pointer: a wheel or a pinch of a frontend.
    #[must_use]
    pub fn zoomed_at(self, steps: f64, x: f64, y: f64, width: f64, height: f64) -> Self {
        let target = clamp_zoom(self.zoom + finite_or(steps, 0.0));
        let dx = finite_or(x, 0.0) - finite_or(width, 0.0) / 2.0;
        let dy = finite_or(y, 0.0) - finite_or(height, 0.0) / 2.0;
        let (px, py) = project(self.lon, self.lat, self.zoom);
        // The world point under the pointer, in world pixels of the old zoom.
        let (lon, lat) = unproject(px + dx, py + dy, self.zoom);
        let (qx, qy) = project(lon, lat, target);
        let (cx, cy) = (qx - dx, qy - dy);
        let (lon, lat) = unproject(cx, cy, target);
        Self::new(lon, lat, target)
    }

    /// Pixels `(x, y)` of a place on a view `width` x `height` whose centre is this position, on the
    /// copy of the world nearest to the centre, so a place across the antimeridian is still beside it.
    pub fn to_screen(&self, lon: f64, lat: f64, width: f64, height: f64) -> (f64, f64) {
        let (cx, cy) = project(self.lon, self.lat, self.zoom);
        let (px, py) = project(wrap_lon(lon), lat, self.zoom);
        let world = super::mercator::world_size(self.zoom);
        let mut dx = px - cx;
        if dx > world / 2.0 {
            dx -= world;
        } else if dx < -world / 2.0 {
            dx += world;
        }
        (
            finite_or(width, 0.0) / 2.0 + dx,
            finite_or(height, 0.0) / 2.0 + (py - cy),
        )
    }

    /// The longitude and latitude under the pixel `(x, y)` of a view `width` x `height`.
    pub fn from_screen(&self, x: f64, y: f64, width: f64, height: f64) -> (f64, f64) {
        let (cx, cy) = project(self.lon, self.lat, self.zoom);
        let dx = finite_or(x, 0.0) - finite_or(width, 0.0) / 2.0;
        let dy = finite_or(y, 0.0) - finite_or(height, 0.0) / 2.0;
        let (lon, lat) = unproject(cx + dx, cy + dy, self.zoom);
        (wrap_lon(lon), lat)
    }

    /// The position that shows the box `west, south, east, north` whole in a view `width` x
    /// `height` (across the antimeridian if `west > east`), at most `max_zoom`. A box that is a
    /// point shows it at `max_zoom`.
    #[must_use]
    pub fn fitting(
        (west, south, east, north): (f64, f64, f64, f64),
        width: f64,
        height: f64,
        max_zoom: f64,
    ) -> Self {
        let span = span_degrees(west, east);
        let centre_lon = wrap_lon(west + span / 2.0);
        let (south, north) = (clamp_lat(south.min(north)), clamp_lat(north.max(south)));
        let (_, top) = project(0.0, north, 0.0);
        let (_, bottom) = project(0.0, south, 0.0);
        let rows = (bottom - top).max(0.0);
        let columns = span / 360.0 * super::mercator::TILE_SIZE;
        let fit = |pixels: f64, needed: f64| {
            if needed <= 0.0 {
                MAX_ZOOM
            } else {
                (finite_or(pixels, 0.0).max(1.0) / needed).log2()
            }
        };
        let zoom = fit(width, columns).min(fit(height, rows));
        let (_, mid) = unproject(0.0, f64::midpoint(top, bottom), 0.0);
        Self::new(centre_lon, mid, zoom.min(max_zoom))
    }
}
