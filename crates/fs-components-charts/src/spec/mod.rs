//! Reading and validating a Vega-Lite spec (no GUI, no data work).
//!
//! The accepted subset is small and provisional (see `DESIGN.md`): marks `bar`, `line`, `area`,
//! `point`, `arc`; channels `x`, `y`, `color` (nominal), `theta`; inline data; no transforms.
//! Everything else is a [`SpecError`] that names the feature. Nothing is ignored.
//!
//! Pattern: Facade — [`parse`] is the entry point; `channel`, `mark`, `check` do the parts.

mod channel;
mod check;
mod error;
mod field;
mod json;
mod mark;
mod model;
mod parse;

pub use error::{ErrorKind, Result, SpecError};
pub use field::lookup;
pub use model::{
    Aggregate, AxisDef, Channel, ChannelDef, Encoding, FieldType, MarkDef, MarkKind, ScaleDef,
    Sort, Spec, StackMode, Title,
};
pub use parse::{from_value, parse};
