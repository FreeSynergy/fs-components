//! The validated spec: only what the subset accepts, typed.
//!
//! Pattern: Value objects — plain data, built by `parse`, read by the layers after it.

use serde_json::{Map, Value};

/// The mark kinds of the subset (provisional).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkKind {
    /// Rectangles from a baseline.
    Bar,
    /// A line through points in order.
    Line,
    /// The area between a line and the baseline.
    Area,
    /// A symbol at each datum.
    Point,
    /// A circle sector (pie, donut).
    Arc,
}

impl MarkKind {
    /// The name Vega-Lite uses.
    pub fn name(self) -> &'static str {
        match self {
            Self::Bar => "bar",
            Self::Line => "line",
            Self::Area => "area",
            Self::Point => "point",
            Self::Arc => "arc",
        }
    }

    /// Line and area connect their data, so an invalid value breaks the path.
    pub fn is_path(self) -> bool {
        matches!(self, Self::Line | Self::Area)
    }
}

/// A mark with its few properties.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarkDef {
    /// Which mark.
    pub kind: MarkKind,
    /// Inner radius of an arc in pixels (0 = pie, more = donut).
    pub inner_radius: f64,
    /// Overlay points on a line or area.
    pub point: bool,
}

/// What kind of values a field holds (Vega-Lite's measurement types).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldType {
    /// Numbers on a continuous scale.
    Quantitative,
    /// Unordered categories.
    Nominal,
    /// Ordered categories.
    Ordinal,
    /// Dates and times.
    Temporal,
}

impl FieldType {
    /// Categories: drawn on a band or point scale.
    pub fn is_discrete(self) -> bool {
        matches!(self, Self::Nominal | Self::Ordinal)
    }
}

/// How rows of a group become one value.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aggregate {
    /// Number of rows.
    Count,
    /// Sum of the valid numbers.
    Sum,
    /// Mean of the valid numbers.
    Mean,
    /// Smallest valid number.
    Min,
    /// Largest valid number.
    Max,
}

/// The title of an axis or legend.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Title {
    /// Made from the field name (and aggregate).
    #[default]
    Auto,
    /// Written in the spec.
    Text(String),
    /// Switched off (`null`).
    Hidden,
}

/// Order of categories.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Default)]
pub enum Sort {
    /// Ascending by value (the Vega-Lite default).
    #[default]
    Ascending,
    /// Descending by value.
    Descending,
    /// As they first appear in the data (`null`).
    DataOrder,
    /// Exactly this list; values not in it are placed after it.
    Values(Vec<Value>),
    /// By the value of the other positional channel, summed per category.
    ByChannel {
        /// `true` for `-x` / `-y`.
        descending: bool,
        /// The channel sorted by.
        channel: Channel,
    },
}

/// Stacking of a quantitative channel.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StackMode {
    /// Stacked from zero (the default where stacking applies).
    #[default]
    Zero,
    /// Stacked and scaled to 0..1 per category.
    Normalize,
    /// Not stacked: marks overlap.
    Off,
}

/// The channels of the subset.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    /// Horizontal position.
    X,
    /// Vertical position.
    Y,
    /// Series colour (nominal only).
    Color,
    /// Angle of an arc.
    Theta,
}

impl Channel {
    /// The name Vega-Lite uses.
    pub fn name(self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
            Self::Color => "color",
            Self::Theta => "theta",
        }
    }
}

/// Axis properties of the subset.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct AxisDef {
    /// `false` when written `"axis": null`.
    pub visible: bool,
    /// Title.
    pub title: Title,
    /// Grid lines (default: on for continuous scales).
    pub grid: Option<bool>,
    /// Tick labels.
    pub labels: bool,
}

impl Default for AxisDef {
    fn default() -> Self {
        Self {
            visible: true,
            title: Title::Auto,
            grid: None,
            labels: true,
        }
    }
}

/// Scale properties of the subset.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ScaleDef {
    /// Include zero in the domain.
    pub zero: Option<bool>,
    /// Extend the domain to round numbers.
    pub nice: Option<bool>,
    /// Fixed domain `[min, max]` of a quantitative field.
    pub domain: Option<[f64; 2]>,
}

/// One channel's definition.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct ChannelDef {
    /// Field name; `None` only for `count`.
    pub field: Option<String>,
    /// Measurement type.
    pub ftype: FieldType,
    /// Aggregate, if any.
    pub aggregate: Option<Aggregate>,
    /// Title (axis or legend).
    pub title: Title,
    /// Axis (x, y).
    pub axis: AxisDef,
    /// Scale (x, y).
    pub scale: ScaleDef,
    /// Category order.
    pub sort: Sort,
    /// Stacking as written (`None` when the spec does not say).
    pub stack: Option<StackMode>,
    /// Legend visible (colour).
    pub legend: bool,
}

/// The encoding: which field goes to which channel.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Encoding {
    /// Horizontal position.
    pub x: Option<ChannelDef>,
    /// Vertical position.
    pub y: Option<ChannelDef>,
    /// Series colour.
    pub color: Option<ChannelDef>,
    /// Angle.
    pub theta: Option<ChannelDef>,
}

impl Encoding {
    /// The definition of a channel.
    pub fn get(&self, c: Channel) -> Option<&ChannelDef> {
        match c {
            Channel::X => self.x.as_ref(),
            Channel::Y => self.y.as_ref(),
            Channel::Color => self.color.as_ref(),
            Channel::Theta => self.theta.as_ref(),
        }
    }
}

/// A whole validated spec.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct Spec {
    /// Major version of the schema named by `$schema` (5 or 6; 6 when there is none).
    pub schema_major: u8,
    /// Title of the chart.
    pub title: Option<String>,
    /// Description: the text alternative written by the spec's author.
    pub description: Option<String>,
    /// Plot width in pixels, if the spec asks for one.
    pub width: Option<f64>,
    /// Plot height in pixels, if the spec asks for one.
    pub height: Option<f64>,
    /// The mark.
    pub mark: MarkDef,
    /// The encoding.
    pub encoding: Encoding,
    /// The inline rows.
    pub data: Vec<Map<String, Value>>,
}
