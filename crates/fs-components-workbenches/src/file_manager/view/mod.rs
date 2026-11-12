//! The file manager as elements.
//!
//! Pattern: none (declares modules and re-exports only) — the view functions read a snapshot of the state and report through callbacks.

mod actions;
mod band;
mod band_size;
mod command_items;
mod commands;
mod frame;
mod item_bar;
mod nav;
mod places;
mod side;
mod side_more;
mod sidebar;
mod styles;
mod tabs;
mod toolbar;
mod work;

pub use actions::OnPick;
pub(super) use actions::{FileActions, OnPick as PickHandler};
pub(super) use band::band;
pub use blinc_layout::prelude::ElementBuilder as Elem;
pub(super) use commands::command_row;
pub(super) use frame::{frame, sizes};
pub(super) use nav::nav_row;
pub(super) use places::{notice_line, status_line};
pub(super) use side::side;
pub(super) use sidebar::tree_pane;
pub use styles::styles;
pub(super) use tabs::tabs_row;
pub(super) use work::work;
