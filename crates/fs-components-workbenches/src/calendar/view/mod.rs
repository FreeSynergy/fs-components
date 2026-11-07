//! The look of the calendar workbench: pure functions from a snapshot of the state to elements.
//!
//! Pattern: Adapter (state to Blinc elements) — each function draws one area and reports what the
//! person does through [`CalendarActions`]; nothing here changes state itself. No function here is
//! ever called with a lock held.

mod actions;
mod detail;
mod detail_bar;
mod form;
mod frame;
mod problems;
mod styles;
mod toolbar;

pub(super) use actions::{Act, ActOn, CalendarActions};
pub(super) use frame::{Places, workbench};
pub use styles::styles;
