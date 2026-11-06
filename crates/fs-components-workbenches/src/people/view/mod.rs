//! The look of the people workbench: pure functions from a snapshot of the state to elements.
//!
//! Pattern: Adapter (state to Blinc elements) — each function draws one area and reports what the
//! person does through [`PeopleActions`]; nothing here changes state itself. No function here is
//! ever called with a lock held.

mod actions;
mod detail;
mod form;
mod frame;
mod list;
mod styles;

pub(super) use actions::PeopleActions;
pub(super) use detail::{button_for, detail};
pub(super) use form::InputSlots;
pub(super) use frame::{Areas, frame, list_size};
pub(super) use list::list;
pub use styles::styles;
