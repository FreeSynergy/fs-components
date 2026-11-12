//! `plan`: the widgets of the two plan screens of the tasks program: a board of tinted columns (A) and a list of
//! the day in the style of a quiet to-do list (B). New in v0.0.13, additive: nothing of an older widget changed.
//!
//! Pattern: none — this file only declares modules and re-exports the public API. Every widget takes plain data
//! and callbacks (the host owns the data and every decision); the sheet [`STYLES`] holds the look, in its own
//! sizes (they are larger than the compact widgets of the older programs), so no other program changes.

mod board;
mod card;
mod chip;
mod entry;
mod events;
mod line;
mod list;
mod model;
mod side;
mod styles;
mod tabs;

pub use board::{BoardView, board_layout, message};
pub use card::{PlanCard, initials_circle};
pub use chip::{Chip, Tone, chip_element};
pub use entry::{Entry, submitted};
pub use events::{Events, redraw};
pub use line::TaskLine;
pub use list::{ListScreen, Section};
pub use model::{Column, Detail, Panel, Tab};
pub use side::{SideItem, SideList};
pub use styles::STYLES;
pub use tabs::tabs_element;
