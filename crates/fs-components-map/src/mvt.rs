//! Vector tiles (MVT, Mapbox Vector Tile 2.1) into the neutral model of the map view.
//!
//! Pattern: Adapter over `mvt-reader` + Strategy for the look of a layer (`style`): the layer names of the common
//! schemas (Protomaps, Shortbread: `water`, `earth`, `landuse`, `buildings`, `roads`, `boundaries`, `places`) pick a tone
//! of the theme. A layer this does not know is drawn plainly, never refused.

use geo_types::Geometry;
use mvt_reader::Reader;

use crate::guard::scan;
use crate::model::{Area, Dot, Label, Line, Paint, TileGeometry, Tone};

/// Why a tile could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MvtError(pub String);

impl std::fmt::Display for MvtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unreadable vector tile: {}", self.0)
    }
}

impl std::error::Error for MvtError {}

/// The look of one layer.
struct Style {
    fill: Option<Paint>,
    line: Option<(Paint, f32)>,
    label: bool,
}

fn style(layer: &str) -> Style {
    let n = |fill, line, label| Style { fill, line, label };
    match layer {
        "water" | "ocean" | "water_polygons" => {
            n(Some(Paint::Theme(Tone::Info, 0.45)), None, false)
        }
        "earth" | "land" | "landuse" | "landcover" | "park" => {
            n(Some(Paint::Theme(Tone::Success, 0.12)), None, false)
        }
        "buildings" | "building" => n(
            Some(Paint::Theme(Tone::TextMuted, 0.25)),
            Some((Paint::Theme(Tone::Border, 0.6), 0.5)),
            false,
        ),
        "roads" | "transportation" | "streets" => {
            n(None, Some((Paint::Theme(Tone::TextMuted, 0.8), 1.2)), false)
        }
        "boundaries" | "boundary" => n(None, Some((Paint::Theme(Tone::Border, 1.0), 1.0)), false),
        "places" | "place" | "place_labels" => n(None, None, true),
        _ => n(
            Some(Paint::Theme(Tone::Surface, 0.3)),
            Some((Paint::Theme(Tone::Border, 0.5), 0.5)),
            false,
        ),
    }
}

fn ring(line: &geo_types::LineString<f32>) -> Vec<[f32; 2]> {
    line.coords().map(|c| [c.x, c.y]).collect()
}

fn add(g: &Geometry<f32>, st: &Style, name: Option<&str>, out: &mut TileGeometry) {
    match g {
        Geometry::Point(p) => add_point(p.x(), p.y(), st, name, out),
        Geometry::MultiPoint(m) => m
            .iter()
            .for_each(|p| add_point(p.x(), p.y(), st, name, out)),
        Geometry::LineString(l) => add_line(l, st, out),
        Geometry::MultiLineString(m) => m.iter().for_each(|l| add_line(l, st, out)),
        Geometry::Polygon(p) => add_polygon(p, st, out),
        Geometry::MultiPolygon(m) => m.iter().for_each(|p| add_polygon(p, st, out)),
        Geometry::GeometryCollection(c) => c.iter().for_each(|g| add(g, st, name, out)),
        _ => {}
    }
}

fn add_point(x: f32, y: f32, st: &Style, name: Option<&str>, out: &mut TileGeometry) {
    if st.label {
        if let Some(text) = name {
            out.labels.push(Label {
                at: [x, y],
                text: text.to_owned(),
                paint: Paint::tone(Tone::Text),
                size: 12.0,
            });
        }
    } else {
        out.dots.push(Dot {
            at: [x, y],
            paint: Paint::tone(Tone::TextMuted),
            radius: 2.0,
        });
    }
}

fn add_line(l: &geo_types::LineString<f32>, st: &Style, out: &mut TileGeometry) {
    if let Some((paint, width)) = st.line {
        out.lines.push(Line {
            points: ring(l),
            paint,
            width,
        });
    }
}

fn add_polygon(p: &geo_types::Polygon<f32>, st: &Style, out: &mut TileGeometry) {
    if let Some(fill) = st.fill {
        let mut rings = vec![ring(p.exterior())];
        rings.extend(p.interiors().iter().map(ring));
        out.areas.push(Area {
            rings,
            fill,
            outline: st.line,
        });
    } else if let Some((paint, width)) = st.line {
        out.lines.push(Line {
            points: ring(p.exterior()),
            paint,
            width,
        });
    }
}

/// The extent of a layer that gives none.
const DEFAULT_EXTENT: u32 = 4096;

/// Decode one tile (uncompressed MVT). The tile is looked at first (`guard`): one over a limit of bytes, of features or
/// of what `mvt-reader` would reserve is refused. The result has the extent of the widest layer; the coordinates of a
/// layer with another extent are scaled to it, so every layer lies on the tile where it belongs.
///
/// # Errors
/// [`MvtError`] when the bytes are not a vector tile or are over a limit.
pub fn decode_tile(bytes: &[u8]) -> Result<TileGeometry, MvtError> {
    let scanned = scan(bytes)?;
    let reader = Reader::new(bytes.to_vec()).map_err(|e| MvtError(format!("{e:?}")))?;
    let layers = reader
        .get_layer_metadata()
        .map_err(|e| MvtError(format!("{e:?}")))?;
    let extent_of = |l: &mvt_reader::layer::Layer| {
        scanned
            .extents
            .get(l.layer_index)
            .copied()
            .flatten()
            .map_or(DEFAULT_EXTENT, |e| e.max(1)) as f32
    };
    let widest = layers.iter().map(extent_of).fold(0.0, f32::max);
    let mut out = TileGeometry::new(if widest > 0.0 { widest } else { 4096.0 });
    for layer in &layers {
        let st = style(&layer.name);
        let features = reader
            .get_features(layer.layer_index)
            .map_err(|e| MvtError(format!("{e:?}")))?;
        let before = Marks::of(&out);
        for f in features {
            let name = match f.properties.as_ref().and_then(|p| p.get("name")) {
                Some(mvt_reader::feature::Value::String(s)) => Some(s.as_str()),
                _ => None,
            };
            add(&f.geometry, &st, name, &mut out);
        }
        let k = out.extent / extent_of(layer);
        before.scale_to(&mut out, k);
    }
    Ok(out)
}

/// How much of a tile there was before a layer was added, to scale only what the layer added.
struct Marks {
    areas: usize,
    lines: usize,
    dots: usize,
    labels: usize,
}

impl Marks {
    fn of(t: &TileGeometry) -> Self {
        Self {
            areas: t.areas.len(),
            lines: t.lines.len(),
            dots: t.dots.len(),
            labels: t.labels.len(),
        }
    }

    fn scale_to(&self, t: &mut TileGeometry, k: f32) {
        if (k - 1.0).abs() < f32::EPSILON {
            return;
        }
        let up = |p: &mut [f32; 2]| {
            p[0] *= k;
            p[1] *= k;
        };
        t.areas[self.areas..]
            .iter_mut()
            .flat_map(|a| a.rings.iter_mut().flatten())
            .for_each(up);
        t.lines[self.lines..]
            .iter_mut()
            .flat_map(|l| l.points.iter_mut())
            .for_each(up);
        t.dots[self.dots..].iter_mut().for_each(|d| up(&mut d.at));
        t.labels[self.labels..]
            .iter_mut()
            .for_each(|l| up(&mut l.at));
    }
}
