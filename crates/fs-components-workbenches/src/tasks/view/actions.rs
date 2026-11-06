//! What the person can do, as callbacks.
//!
//! Pattern: Command-like callbacks (Observer) — the widget decides what each one does: change the
//! state and draw again.

use std::sync::Arc;

use blinc_cn::cn;
use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};

/// A plain action.
pub(in crate::tasks) type Act = Arc<dyn Fn() + Send + Sync>;
/// An action about something named by a key.
pub(in crate::tasks) type ActOn = Arc<dyn Fn(&str) + Send + Sync>;
/// The person gave a field this value.
pub(in crate::tasks) type Typed = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// The things the view reports.
#[derive(Clone)]
pub(in crate::tasks) struct TaskActions {
    /// A row was chosen.
    pub select: ActOn,
    /// The keys of the list went to a row.
    pub walk: ActOn,
    /// Show more rows.
    pub more: Act,
    /// A new task.
    pub create: Act,
    /// Start editing.
    pub edit: Act,
    /// Save.
    pub save: Act,
    /// Cancel the edit.
    pub cancel: Act,
    /// Go on editing after being asked.
    pub keep_editing: Act,
    /// Really discard the changes.
    pub confirm_discard: Act,
    /// The first press of Delete: asks.
    pub ask_delete: Act,
    /// Really delete.
    pub confirm_delete: Act,
    /// Keep the task after being asked.
    pub keep: Act,
    /// Show the list.
    pub view_list: Act,
    /// Show the status view.
    pub view_status: Act,
    /// Open the card under the keyboard's cursor.
    pub open_cursor: Act,
    /// Move the task shown to this column.
    pub move_selected: ActOn,
    /// A choice was pressed in the form: draws again.
    pub choose: Typed,
    /// Text typed: does not draw again.
    pub typed: Typed,
}

/// A button with an element id on a wrapper, so a host or a test can find it.
pub(super) fn button(id: String, label: &str, variant: ButtonVariant, act: &Act) -> Div {
    let act = Arc::clone(act);
    div().id(id).w_fit().child(
        cn::button(label.to_string())
            .variant(variant)
            .on_click(move |_| act()),
    )
}
