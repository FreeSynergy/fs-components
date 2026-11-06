//! Scales: from data values to pixel positions and tick marks (pure, exact numbers).
//!
//! Pattern: Strategy by field type — a linear scale for numbers, a time scale (calendar ticks in
//! the zone of the data) for dates, a band or point scale for categories.

pub mod band;
pub mod format;
pub mod linear;
pub mod ticks;
pub mod timeticks;

pub use band::Band;
pub use linear::{DomainRule, Linear, domain};
