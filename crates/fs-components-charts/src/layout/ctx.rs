//! What every mark layout reads: the spec, the cleaned data, the words, the series.
//!
//! Pattern: Parameter object — one borrowed bundle instead of five arguments everywhere.

use super::cats::Cats;
use super::describe;
use crate::data::{Datum, Prepared};
use crate::labels::Labels;
use crate::scene::{Dash, Hit, Shape};
use crate::spec::{Channel, Spec};

/// The inputs of a layout.
pub struct Ctx<'a> {
    /// The validated spec.
    pub spec: &'a Spec,
    /// The cleaned data.
    pub data: &'a Prepared,
    /// The caller's words.
    pub labels: &'a Labels,
    /// The series (colour categories); empty without a colour channel.
    pub series: &'a Cats,
    /// Most marks to emit.
    pub max_marks: usize,
}

impl Ctx<'_> {
    /// The series number of a datum (0 without colour).
    pub fn series_of(&self, d: &Datum) -> usize {
        self.series
            .index_of(&d.cells[Channel::Color.index()], self.labels)
            .unwrap_or(0)
    }

    /// Several series are drawn.
    pub fn multi(&self) -> bool {
        self.series.len() > 1
    }

    /// A hit for a datum.
    pub fn hit(&self, d: &Datum, area: [f32; 4]) -> Hit {
        Hit {
            area,
            text: describe::text(self.spec, d, &self.data.zones, self.labels),
        }
    }
}

/// The line pattern of series `i`.
pub fn dash_of(i: usize) -> Dash {
    [Dash::Solid, Dash::Dashed, Dash::Dotted, Dash::DashDot][i % 4]
}

/// The symbol of series `i`.
pub fn shape_of(i: usize) -> Shape {
    [
        Shape::Circle,
        Shape::Square,
        Shape::Diamond,
        Shape::TriangleUp,
        Shape::Cross,
        Shape::TriangleDown,
    ][i % 6]
}
