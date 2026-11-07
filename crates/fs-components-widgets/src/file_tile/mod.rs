//! `file-tile`: one thing as a tile, and a grid of tiles of which only those in view exist.
//!
//! Pattern: Facade — this file only declares modules and re-exports the public API.

mod grid;
mod styles;
mod tile;

pub use grid::{TileGrid, grid_target};
pub use styles::STYLES;
pub use tile::FileTile;
