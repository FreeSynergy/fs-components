//! The scene: what a chart looks like, as plain shapes with colour *roles*, not colours.
//!
//! Pattern: Intermediate representation (Builder output). Layout produces a [`Scene`]; the view
//! draws it with the theme's colours. A plugin could return the same description and let the host
//! draw it (charts.md, "interface proposal"). Pure data: tested with exact numbers, no GUI.

/// Which colour a shape wants; the view maps it to a theme colour.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Paint {
    /// Series number `n` of the categorical palette.
    Series(usize),
    /// Main text.
    Text,
    /// Less important text.
    Muted,
    /// Grid lines.
    Grid,
    /// Axis lines and ticks.
    Axis,
    /// The surface behind the chart (for gaps between slices).
    Background,
}

/// Line pattern; series differ by pattern as well as by colour. The layout cuts a dashed line
/// into its dashes (see `layout::dash`), so a scene holds only solid paths.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dash {
    /// Unbroken.
    Solid,
    /// Long dashes.
    Dashed,
    /// Dots.
    Dotted,
    /// Dash and dot.
    DashDot,
}

/// Symbol shape; series differ by shape as well as by colour.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// Circle.
    Circle,
    /// Square.
    Square,
    /// Diamond.
    Diamond,
    /// Triangle pointing up.
    TriangleUp,
    /// Cross.
    Cross,
    /// Triangle pointing down.
    TriangleDown,
}

/// The size classes of chart text; pixel sizes come from the metrics.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextSize {
    /// The chart title.
    Title,
    /// Axis and legend titles.
    AxisTitle,
    /// Tick labels, legend entries, notes.
    Label,
}

/// Horizontal anchor of a text.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    /// `x` is the left edge.
    Start,
    /// `x` is the centre.
    Middle,
    /// `x` is the right edge.
    End,
}

/// Vertical anchor of a text.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Baseline {
    /// `y` is the top.
    Top,
    /// `y` is the middle.
    Middle,
    /// `y` is the bottom.
    Bottom,
}

/// One shape. Coordinates are pixels from the top left of the chart.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    /// A filled rectangle.
    Rect {
        /// Left.
        x: f32,
        /// Top.
        y: f32,
        /// Width (never negative).
        w: f32,
        /// Height (never negative).
        h: f32,
        /// Colour role.
        paint: Paint,
        /// 0..1.
        opacity: f32,
    },
    /// A straight line.
    Line {
        /// Start `[x, y]`.
        from: [f32; 2],
        /// End `[x, y]`.
        to: [f32; 2],
        /// Colour role.
        paint: Paint,
        /// Width.
        width: f32,
    },
    /// An open line through points.
    Path {
        /// The points.
        points: Vec<[f32; 2]>,
        /// Colour role.
        paint: Paint,
        /// Width.
        width: f32,
    },
    /// A filled polygon.
    Polygon {
        /// The corners.
        points: Vec<[f32; 2]>,
        /// Colour role.
        paint: Paint,
        /// 0..1.
        opacity: f32,
    },
    /// A symbol.
    Marker {
        /// Centre `[x, y]`.
        at: [f32; 2],
        /// Shape.
        shape: Shape,
        /// Radius of the circle with the same area.
        radius: f32,
        /// Colour role.
        paint: Paint,
        /// Filled, or only the outline.
        filled: bool,
        /// 0..1.
        opacity: f32,
    },
    /// A ring sector; angles in radians from 12 o'clock, clockwise.
    Sector {
        /// Centre `[x, y]`.
        centre: [f32; 2],
        /// Inner radius (0 for a pie).
        inner: f32,
        /// Outer radius.
        outer: f32,
        /// Start angle.
        start: f32,
        /// End angle.
        end: f32,
        /// Colour role.
        paint: Paint,
    },
    /// A text.
    Text {
        /// Anchor point `[x, y]`.
        at: [f32; 2],
        /// The words.
        text: String,
        /// Size class.
        size: TextSize,
        /// Bold.
        bold: bool,
        /// Colour role.
        paint: Paint,
        /// Horizontal anchor.
        anchor: Anchor,
        /// Vertical anchor.
        baseline: Baseline,
        /// Rotation in degrees, counter-clockwise (-90 reads bottom to top... see `Item::Text`).
        angle: f32,
    },
}

/// Something under the pointer: the data behind a mark, for tooltips and hit tests.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    /// Bounding box `[x, y, w, h]`.
    pub area: [f32; 4],
    /// The datum as text, `field: value` per line.
    pub text: String,
}

/// A whole chart.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    /// Width in pixels.
    pub width: f32,
    /// Height in pixels.
    pub height: f32,
    /// Shapes, back to front.
    pub items: Vec<Item>,
    /// Places that carry data.
    pub hits: Vec<Hit>,
    /// The plot area `[x, y, w, h]`.
    pub plot: [f32; 4],
    /// Index of the first mark in `items` (before it: grid, axes, legend, texts).
    pub marks_from: usize,
    /// Marks are cut off at the plot area (the spec fixed a scale domain, as in Vega-Lite).
    pub clip_marks: bool,
    /// Marks left out because there were more than `Metrics::max_marks` (after merging marks
    /// that fall on the same pixel); the chart says so under the plot.
    pub omitted: usize,
}

impl Scene {
    /// The datum under a point, nearest centre first when marks overlap; for tooltips and for
    /// reading a chart by pointing. `slack` widens every mark's box by that many pixels.
    pub fn hit_at(&self, x: f32, y: f32, slack: f32) -> Option<&Hit> {
        self.hits
            .iter()
            .filter(|h| {
                x >= h.area[0] - slack
                    && x <= h.area[0] + h.area[2] + slack
                    && y >= h.area[1] - slack
                    && y <= h.area[1] + h.area[3] + slack
            })
            .min_by(|a, b| {
                let d = |h: &Hit| {
                    (x - h.area[0] - h.area[2] / 2.0).hypot(y - h.area[1] - h.area[3] / 2.0)
                };
                d(a).total_cmp(&d(b))
            })
    }
}
