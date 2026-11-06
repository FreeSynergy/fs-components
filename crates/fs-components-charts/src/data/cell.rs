//! One value of a field, read the way its type says.
//!
//! Pattern: Value object + Strategy by field type — `read` is the one place where a JSON value
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
    /// A number; for a temporal field milliseconds on the UTC line.
    Num(f64),
    /// A category written as text.
    Text(String),
    /// `null` or absent. On a discrete channel this is a category of its own.
    Null,
    /// Cannot be drawn, and why.
    Invalid(Invalid),
}

impl Cell {
    /// The number, if there is one.
    pub fn num(&self) -> Option<f64> {
        match self {
            Self::Num(n) => Some(*n),
            _ => None,
        }
    }

    /// Cannot be put on a continuous scale.
    pub fn is_missing(&self) -> bool {
        !matches!(self, Self::Num(_))
    }
}

/// Read `value` as a field of type `ft`; `zone` collects the zones of temporal texts.
pub fn read(value: Option<&Value>, ft: FieldType, zone: &mut Zones) -> Cell {
    let Some(value) = value else {
        return Cell::Null;
    };
    if value.is_null() {
        return Cell::Null;
    }
    match ft {
        FieldType::Quantitative => match value {
            Value::Number(n) => n
                .as_f64()
                .map_or(Cell::Invalid(Invalid::NotANumber), Cell::Num),
            Value::String(s) => match s.trim().parse::<f64>() {
                Ok(x) if x.is_finite() => Cell::Num(x),
                _ => Cell::Invalid(Invalid::NotANumber),
            },
            _ => Cell::Invalid(Invalid::WrongKind),
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
                Cell::Num(p.ms)
            }),
            Value::Number(n) => n.as_f64().map_or(Cell::Invalid(Invalid::NotADate), |ms| {
                zone.see(Zone::Offset(0));
                Cell::Num(ms)
            }),
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
