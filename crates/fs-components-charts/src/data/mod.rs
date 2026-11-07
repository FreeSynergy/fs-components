//! Turning the rows of a spec into the values a chart draws (pure, no GUI).
//!
//! Pattern: Facade — `column` reads each field by its type, `aggregate` groups, `prepare`
//! drops what cannot be drawn and *counts* it, so nothing disappears silently.

mod aggregate;
pub mod calendar;
mod cell;
mod jsnum;
mod prepare;
mod report;
pub mod time;

pub use cell::{Cell, Invalid, Zones, read};
pub use prepare::{Datum, Prepared, prepare};
pub use report::{Issue, IssueKind, Report};
