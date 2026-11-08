//! The people workbench: a list of people, one person read, edited, created or deleted.
//!
//! Pattern: none (declares modules and re-exports only) — [`PeopleWidget`] is what a program places. Like the reader the workbench is
//! open (ADR 0037): it knows no store and no program core, only [`PeopleSource`], and every word
//! comes from [`PeopleLabels`]. The state machine [`PeopleState`] works without a window.

mod delete;
mod edit;
mod labels;
mod model;
mod shown;
mod source;
mod state;
mod view;
mod widget;
mod widget_actions;

pub use edit::PersonEdit;
pub use labels::PeopleLabels;
pub use model::{
    DraftField, DraftGroup, FieldCheck, PeopleCapabilities, PersonDraft, PersonField, PersonRow,
    PersonSection, PersonSheet,
};
pub use shown::{Outcome, Shown, Variant};
pub use source::PeopleSource;
pub use state::{PAGE, PeopleState};
pub use view::styles;
pub use widget::PeopleWidget;
