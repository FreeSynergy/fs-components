//! The calendar workbench: a month, week, day or agenda, one item read, edited, created or deleted.
//!
//! Pattern: none (declares modules and re-exports only) — [`CalendarWidget`] is what a program places. It embeds the `calendar-view` widget
//! and draws no calendar of its own (ADR 0005). Like the people workbench it is open (ADR 0037): it
//! knows no store and no program core, only [`CalendarSource`], and every word comes from
//! [`CalendarLabels`]. The state machine [`CalendarState`] works without a window.

mod delete;
mod draft;
mod edit;
mod labels;
mod model;
mod navigate;
mod save;
mod shown;
mod shown_read;
mod source;
mod state;
mod view;
mod widget;
mod widget_actions;

pub use draft::{DraftField, FieldKind, ItemDraft};
pub use edit::ItemEdit;
pub use fs_components_widgets::{CalendarItem, CalendarNames, Day, Moment, Picked, ViewRange};
pub use labels::CalendarLabels;
pub use model::{
    CalendarCapabilities, CalendarContext, ItemProblem, ItemSheet, ItemsRead, SheetField,
    SheetSection,
};
pub use shown::{Outcome, Shown, Variant};
pub use source::CalendarSource;
pub use state::{CalendarState, FIRST_YEAR, LAST_YEAR};
pub use view::styles;
pub use widget::CalendarWidget;
