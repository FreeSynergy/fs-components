//! The views of the timeline.
//!
//! Pattern: none — declares the modules and re-exports.

mod arrows;
mod axis;
mod keys;
mod parts;
mod rows;
mod toolbar;
mod whole;

pub use keys::bind_keys;
pub use whole::timeline;
