//! The look of the tasks workbench: pure functions from a snapshot of the state to elements.
//!
//! Pattern: Adapter (state to Blinc elements) — each function draws one area and reports what the
//! person does through [`TaskActions`]; nothing here changes state itself. No function here is ever
//! called with a lock held.

mod actions;
mod board;
mod detail;
mod form;
mod frame;
mod list;
mod styles;
mod toolbar;

pub(super) use actions::{ActOn, TaskActions};
pub(super) use board::board_area;
pub(super) use detail::detail;
pub(super) use frame::{Areas, frame, work_size};
pub(super) use list::list;
pub use styles::styles;
pub(super) use toolbar::toolbar;
