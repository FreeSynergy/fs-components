//! Web Mercator arithmetic: longitude and latitude to world pixels and back.
//!
//! Pattern: Pure functions — no graphics, no state. Every function accepts any `f64`, also NaN and
//! infinities, and answers with a number on the map: a bad value is replaced, never propagated, so
//! a caller cannot make the view panic or blank by passing one.

use std::f64::consts::PI;

/// Pixels of a tile at its own zoom (the size every common tile scheme uses).
pub const TILE_SIZE: f64 = 256.0;
/// The northernmost latitude Web Mercator can show.
pub const MAX_LAT: f64 = 85.051_128_779_806_59;
/// The smallest zoom.
pub const MIN_ZOOM: f64 = 0.0;
/// The largest zoom the view accepts (the same limit as the tile scheme of the core).
pub const MAX_ZOOM: f64 = 22.0;

/// `value` if it is a finite number, else `fallback`.
pub fn finite_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() { value } else { fallback }
}

/// A zoom inside the limits; a number that is not one is the lowest zoom.
#[must_use]
pub fn clamp_zoom(zoom: f64) -> f64 {
    finite_or(zoom, MIN_ZOOM).clamp(MIN_ZOOM, MAX_ZOOM)
}

/// A latitude Web Mercator can show; a number that is not one is the equator.
#[must_use]
pub fn clamp_lat(lat: f64) -> f64 {
    finite_or(lat, 0.0).clamp(-MAX_LAT, MAX_LAT)
}

/// A longitude in `[-180, 180)`: 190 is -170. A number that is not one is the meridian.
#[must_use]
pub fn wrap_lon(lon: f64) -> f64 {
    let lon = finite_or(lon, 0.0);
    if (-180.0..180.0).contains(&lon) {
        return lon;
    }
    let wrapped = (lon + 180.0).rem_euclid(360.0) - 180.0;
    // `rem_euclid` can give exactly 360.0 for a tiny negative input.
    if wrapped >= 180.0 { -180.0 } else { wrapped }
}

/// Side of the whole world in pixels at `zoom`.
#[must_use]
pub fn world_size(zoom: f64) -> f64 {
    TILE_SIZE * 2f64.powf(clamp_zoom(zoom))
}

/// World pixels `(x, y)` of a position at `zoom`: `x` grows eastwards from -180, `y` southwards
/// from the northern limit. Latitude is clamped; longitude is not wrapped (so a caller can place a
/// point on the copy of the world nearest to the view).
#[must_use]
pub fn project(lon: f64, lat: f64, zoom: f64) -> (f64, f64) {
    let size = world_size(zoom);
    let lon = finite_or(lon, 0.0).clamp(-1.0e6, 1.0e6);
    let sin = clamp_lat(lat).to_radians().sin();
    let x = (lon + 180.0) / 360.0 * size;
    let y = (0.5 - ((1.0 + sin) / (1.0 - sin)).ln() / (4.0 * PI)) * size;
    (x, y)
}

/// The longitude and latitude at world pixels `(x, y)` at `zoom`; the latitude is clamped, the
/// longitude is not wrapped.
#[must_use]
pub fn unproject(x: f64, y: f64, zoom: f64) -> (f64, f64) {
    let size = world_size(zoom);
    let x = finite_or(x, size / 2.0).clamp(-1.0e15, 1.0e15);
    let y = finite_or(y, size / 2.0).clamp(-1.0e15, 1.0e15);
    let lon = x / size * 360.0 - 180.0;
    let n = PI - 2.0 * PI * y / size;
    let lat = (0.5 * (n.exp() - (-n).exp())).atan().to_degrees();
    (lon, clamp_lat(lat))
}

/// Is the span `west..east` one that crosses the antimeridian (west is east of east)?
#[must_use]
pub fn crosses_antimeridian(west: f64, east: f64) -> bool {
    finite_or(west, 0.0) > finite_or(east, 0.0)
}

/// The width in degrees of the span `west..east`, across the antimeridian if `west > east`.
#[must_use]
pub fn span_degrees(west: f64, east: f64) -> f64 {
    let (west, east) = (finite_or(west, 0.0), finite_or(east, 0.0));
    if west > east {
        east - west + 360.0
    } else {
        east - west
    }
}
