//! What the caller gives the map to draw: decoded geometry per tile, in a neutral model.
//!
//! Pattern: Value objects — polygons, lines, points and labels in the coordinates of one tile, each
//! with a [`Paint`] the caller chose. The widget knows no tile format and no data source; a decoder
//! in the caller's crate fills these. Colours are theme tones or the caller's own RGBA, never the
//! widget's.

/// A colour of the theme, by what it is for.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tone {
    /// The window's background.
    Background,
    /// A surface.
    Surface,
    /// A raised surface.
    SurfaceElevated,
    /// A border or a thin line.
    Border,
    /// Ordinary text.
    Text,
    /// Quiet text.
    TextMuted,
    /// The main accent.
    Primary,
    /// The second accent.
    Secondary,
    /// Another accent.
    Accent,
    /// Information (blue in most themes).
    Info,
    /// Success (green in most themes).
    Success,
    /// A warning.
    Warning,
    /// An error.
    Error,
}

/// How something is painted: a tone of the theme with an opacity, or the caller's own colour.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Paint {
    /// A colour of the theme, with an opacity between 0 and 1.
    Theme(Tone, f32),
    /// The caller's colour: red, green, blue, alpha.
    Rgba(u8, u8, u8, u8),
}

impl Paint {
    /// A tone, fully opaque.
    #[must_use]
    pub fn tone(tone: Tone) -> Self {
        Self::Theme(tone, 1.0)
    }
}

/// A point in the coordinates of its tile: `0..extent` across the tile (a little beyond is allowed:
/// tiles carry a buffer), y downwards.
pub type TilePoint = [f32; 2];

/// An area: the first ring is the outline, the others are holes (as in `GeoJSON` and MVT).
#[derive(Clone, Debug, PartialEq)]
pub struct Area {
    /// Outline first, then holes; each ring is closed implicitly.
    pub rings: Vec<Vec<TilePoint>>,
    /// How it is filled.
    pub fill: Paint,
    /// An outline drawn around it, with its width in pixels.
    pub outline: Option<(Paint, f32)>,
}

/// A line.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// The points in order.
    pub points: Vec<TilePoint>,
    /// How it is drawn.
    pub paint: Paint,
    /// Width in pixels.
    pub width: f32,
}

/// A dot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dot {
    /// Where.
    pub at: TilePoint,
    /// How it is drawn.
    pub paint: Paint,
    /// Radius in pixels.
    pub radius: f32,
}

/// A word on the map.
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    /// Where its middle is.
    pub at: TilePoint,
    /// The text (shared: a feature with many points holds it once).
    pub text: std::sync::Arc<str>,
    /// How it is drawn.
    pub paint: Paint,
    /// Size in pixels.
    pub size: f32,
}

/// Everything drawn from one tile. Primitives are drawn in this order: areas, lines, dots, labels;
/// within each kind, in the order given (so the caller orders layers).
#[derive(Clone, Debug, PartialEq)]
pub struct TileGeometry {
    /// How many units a tile is wide in the coordinates of the points (4096 for a vector tile).
    pub extent: f32,
    /// Areas.
    pub areas: Vec<Area>,
    /// Lines.
    pub lines: Vec<Line>,
    /// Dots.
    pub dots: Vec<Dot>,
    /// Labels.
    pub labels: Vec<Label>,
}

impl TileGeometry {
    /// About how many bytes this tile holds in memory: the points (8 bytes each), the structs that carry them and the
    /// text of the labels (counted for every label, though a feature's points share one copy: an upper bound).
    #[must_use]
    pub fn memory(&self) -> usize {
        use std::mem::size_of;
        let point = size_of::<TilePoint>();
        let areas: usize = self
            .areas
            .iter()
            .map(|a| {
                size_of::<Area>()
                    + a.rings
                        .iter()
                        .map(|r| size_of::<Vec<TilePoint>>() + r.len() * point)
                        .sum::<usize>()
            })
            .sum();
        let lines: usize = self
            .lines
            .iter()
            .map(|l| size_of::<Line>() + l.points.len() * point)
            .sum();
        let labels: usize = self
            .labels
            .iter()
            .map(|l| size_of::<Label>() + l.text.len())
            .sum();
        size_of::<Self>() + areas + lines + self.dots.len() * size_of::<Dot>() + labels
    }

    /// An empty tile of the given extent.
    #[must_use]
    pub fn new(extent: f32) -> Self {
        Self {
            extent: if extent.is_finite() && extent > 0.0 {
                extent
            } else {
                1.0
            },
            areas: Vec::new(),
            lines: Vec::new(),
            dots: Vec::new(),
            labels: Vec::new(),
        }
    }

    /// How many primitives the tile holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.areas.len() + self.lines.len() + self.dots.len() + self.labels.len()
    }

    /// Is the tile empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A marker on the map.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct Marker {
    /// What the caller calls it; reported back when it is chosen.
    pub key: String,
    /// Longitude.
    pub lon: f64,
    /// Latitude.
    pub lat: f64,
    /// The word beside it, if any. Never given for a marker that is `marked`.
    pub label: Option<String>,
    /// Marked as needing care (ADR 0029): drawn differently and never labelled.
    pub marked: bool,
    /// The one chosen.
    pub selected: bool,
}

impl Marker {
    /// A marker at a position.
    pub fn new(key: impl Into<String>, lon: f64, lat: f64) -> Self {
        Self {
            key: key.into(),
            lon,
            lat,
            label: None,
            marked: false,
            selected: false,
        }
    }

    /// With a word beside it; a marked marker takes none.
    #[must_use]
    pub fn labelled(mut self, label: impl Into<String>) -> Self {
        if !self.marked {
            self.label = Some(label.into());
        }
        self
    }

    /// Marked as needing care.
    #[must_use]
    pub fn marked(mut self) -> Self {
        self.marked = true;
        self.label = None;
        self
    }

    /// The one chosen.
    #[must_use]
    pub fn selected(mut self) -> Self {
        self.selected = true;
        self
    }
}
