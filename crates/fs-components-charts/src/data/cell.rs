//! One value of a field, read the way its type says.
//!
//! Pattern: Value object (`Cell` is a tagged value, valid on construction; `read` is the one place that makes it) — `read` is the one place where a JSON value
//! becomes a number, a category, an instant, or a *named* reason why it cannot be drawn.

use serde_json::Value;

use super::time::{self, Zone};
use crate::spec::FieldType;

/// Why a value cannot be drawn.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Invalid {
    /// A text that is not a finite number (`"NaN"`, `"abc"`, `""`, `"Infinity"`).
    NotANumber,
    /// A text that is not a date, or an impossible date.
    NotADate,
    /// An object, array or boolean where a number or date was expected.
    WrongKind,
    /// A result that does not fit a number (a sum of huge values).
    Overflow,
    /// A negative value where the mark cannot show one (an arc's angle).
    Negative,
}

/// One value.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
    /// A number.
    Num(f64),
    /// An instant: milliseconds on the UTC line and the zone the data wrote it in.
    Time {
        /// Milliseconds since 1970-01-01T00:00 UTC (as if UTC for a wall-clock time).
        ms: f64,
        /// The zone of the text.
        zone: Zone,
    },
    /// A category written as text.
    Text(String),
    /// `null`. On a discrete channel this is a category of its own.
    Null,
    /// The field is not in the row at all. On a discrete channel a category of its own, after
    /// `null` (Vega-Lite keeps `null` and `undefined` apart).
    Absent,
    /// Cannot be drawn, and why.
    Invalid(Invalid),
}

impl Cell {
    /// The number, if there is one.
    pub fn num(&self) -> Option<f64> {
        match self {
            Self::Num(n) | Self::Time { ms: n, .. } => Some(*n),
            _ => None,
        }
    }

    /// Cannot be put on a continuous scale.
    pub fn is_missing(&self) -> bool {
        !matches!(self, Self::Num(_) | Self::Time { .. })
    }
}

/// The latest and earliest instant of a JavaScript date, in milliseconds.
const MAX_MS: f64 = 8.64e15;

/// Read `value` as a field of type `ft`; `zone` collects the zones of temporal texts.
pub fn read(value: Option<&Value>, ft: FieldType, zone: &mut Zones) -> Cell {
    let Some(value) = value else {
        return Cell::Absent;
    };
    if value.is_null() {
        return Cell::Null;
    }
    match ft {
        FieldType::Quantitative => match (super::jsnum::number(value), value) {
            (Some(x), _) => Cell::Num(x),
            (None, Value::String(_) | Value::Number(_)) => Cell::Invalid(Invalid::NotANumber),
            (None, _) => Cell::Invalid(Invalid::WrongKind),
        },
        FieldType::Nominal | FieldType::Ordinal => match value {
            Value::String(s) => Cell::Text(s.clone()),
            Value::Number(n) => n
                .as_f64()
                .map_or(Cell::Invalid(Invalid::NotANumber), Cell::Num),
            Value::Bool(b) => Cell::Text(b.to_string()),
            _ => Cell::Invalid(Invalid::WrongKind),
        },
        FieldType::Temporal => match value {
            Value::String(s) => time::parse(s).map_or(Cell::Invalid(Invalid::NotADate), |p| {
                zone.see(p.zone);
                Cell::Time {
                    ms: p.ms,
                    zone: p.zone,
                }
            }),
            // JavaScript dates reach 8.64e15 ms either side of 1970; beyond that is no date
            Value::Number(n) => match n.as_f64() {
                Some(ms) if ms.abs() <= MAX_MS => {
                    zone.see(Zone::Offset(0));
                    Cell::Time {
                        ms,
                        zone: Zone::Offset(0),
                    }
                }
                _ => Cell::Invalid(Invalid::NotADate),
            },
            _ => Cell::Invalid(Invalid::WrongKind),
        },
    }
}

/// The zones seen in a temporal field: the first offset names the axis; mixing is detected.
#[derive(Clone, Copy, Debug, Default)]
pub struct Zones {
    first: Option<Zone>,
    /// Naive and offset times were both seen.
    pub mixed: bool,
}

impl Zones {
    fn see(&mut self, z: Zone) {
        match self.first {
            None => self.first = Some(z),
            Some(f) => self.mixed |= (f == Zone::Naive) != (z == Zone::Naive),
        }
    }

    /// The zone the axis is drawn in: the first one seen (`Naive` if there was none).
    pub fn axis(&self) -> Zone {
        self.first.unwrap_or(Zone::Naive)
    }
}
