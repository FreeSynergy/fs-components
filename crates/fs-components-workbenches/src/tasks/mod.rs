//! The tasks workbench: one set of tasks as a list or by status, one task read, edited, created or
//! deleted, and moved from one status to another.
//!
//! Pattern: none (declares modules and re-exports only) — [`TasksWidget`] is what a host places. The workbench knows no store and no
//! program core, only [`TaskSource`], and every word comes from [`TaskLabels`]. The state machine
//! [`TasksState`] works without a window. A workbench does not run alone (ADR 0056): it has no
//! window function, and gives its host what is current as a plain [`TasksContext`].

mod board;
mod context;
mod delete;
mod draft;
mod edit;
mod labels;
mod model;
mod moving;
mod shown;
mod source;
mod state;
mod view;
mod widget;
mod widget_actions;

pub use context::{TaskView, TasksContext};
pub use draft::{DraftField, FieldKind, FieldOption, TaskDraft};
pub use edit::TaskEdit;
pub use fs_components_widgets::sortable_list::{Lane, Outcome as BoardOutcome, Place};
pub use labels::TaskLabels;
pub use model::{
    RowQuery, TaskCapabilities, TaskColumn, TaskField, TaskRow, TaskSection, TaskSheet,
};
pub use moving::MoveOutcome;
pub use shown::{Outcome, Shown, Variant};
pub use source::TaskSource;
pub use state::{PAGE, TasksState};
pub use view::styles;
pub use widget::TasksWidget;
