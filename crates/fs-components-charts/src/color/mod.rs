//! Colour: spaces, colour-vision simulation, and the choice of series colours (pure).
//!
//! Pattern: Strategy — `palette::pick` chooses from the theme with `cvd` as the judge.

pub mod cvd;
pub mod palette;
pub mod space;

pub use palette::{MIN_CONTRAST, MIN_DIFFERENCE, SLOTS, pick};
pub use space::Rgb;
