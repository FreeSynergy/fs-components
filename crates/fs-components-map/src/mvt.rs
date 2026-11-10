//! Vector tiles (MVT, Mapbox Vector Tile 2.1) into the neutral model of the map view.
//!
//! Pattern: Adapter over `mvt-reader` + Strategy for the look of a layer (`style`): the layer names of the common
//! schemas (Protomaps, Shortbread: `water`, `earth`, `landuse`, `buildings`, `roads`, `boundaries`, `places`) pick a tone
//! of the theme. A layer this does not know is drawn plainly, never refused.

use geo_types::Geometry;
use mvt_reader::Reader;

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

/// Decode one tile (uncompressed MVT) with the extent the tile says (4096 when the layers give none).
///
/// # Errors
/// [`MvtError`] when the bytes are not a vector tile.
pub fn decode_tile(bytes: &[u8]) -> Result<TileGeometry, MvtError> {
    let reader = Reader::new(bytes.to_vec()).map_err(|e| MvtError(format!("{e:?}")))?;
    let layers = reader
        .get_layer_metadata()
        .map_err(|e| MvtError(format!("{e:?}")))?;
    let extent = layers
        .iter()
        .map(|l| l.extent)
        .max()
        .map_or(4096.0, |e| e as f32);
    let mut out = TileGeometry::new(extent);
    for layer in &layers {
        let st = style(&layer.name);
        let features = reader
            .get_features(layer.layer_index)
            .map_err(|e| MvtError(format!("{e:?}")))?;
        for f in features {
            let name = match f.properties.as_ref().and_then(|p| p.get("name")) {
                Some(mvt_reader::feature::Value::String(s)) => Some(s.as_str()),
                _ => None,
            };
            add(&f.geometry, &st, name, &mut out);
        }
    }
    Ok(out)
}
