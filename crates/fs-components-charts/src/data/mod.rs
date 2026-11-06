//! Turning the rows of a spec into the values a chart draws (pure, no GUI).
//!
//! Pattern: Pipeline — `column` reads each field by its type, `aggregate` groups, `prepare`
//! drops what cannot be drawn and *counts* it, so nothing disappears silently.

mod aggregate;
pub mod calendar;
mod cell;
mod prepare;
mod report;
pub mod time;

pub use cell::{Cell, Invalid, Zones, read};
pub use prepare::{Datum, Prepared, prepare};
pub use report::{Issue, IssueKind, Report};
