//! The views of the grid.
//!
//! Pattern: none — declares the modules and re-exports.

mod body;
mod grid;
mod head;
mod keys;
mod month;
mod parts;
mod timed;
mod toolbar;

pub use grid::grid;
pub use keys::bind_keys;
