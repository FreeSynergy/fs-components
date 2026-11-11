//! The file manager as elements.
//!
//! Pattern: none (declares modules and re-exports only) — the view functions read a snapshot of the state and report through callbacks.

mod actions;
mod band;
mod band_size;
mod frame;
mod item_bar;
mod nav;
mod places;
mod side;
mod side_more;
mod styles;
mod toolbar;
mod work;

pub use actions::OnPick;
pub(super) use actions::{FileActions, OnPick as PickHandler};
pub(super) use band::band;
pub use blinc_layout::prelude::ElementBuilder as Elem;
pub(super) use frame::{frame, sizes};
pub(super) use nav::{command_row, nav_row};
pub(super) use places::{notice_line, status_line, tree_pane};
pub(super) use side::side;
pub use styles::styles;
pub(super) use work::work;
