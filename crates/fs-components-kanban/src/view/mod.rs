//! The views of the board.
//!
//! Pattern: none — declares the modules and re-exports.

mod board;
mod card;
mod column;
mod controls;
mod keys;

pub use board::board;
pub use keys::bind_keys;
