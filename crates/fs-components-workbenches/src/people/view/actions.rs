//! What the person can do, as callbacks.
//!
//! Pattern: Observer — the widget decides what each one does: change the
//! state and draw again.

use std::sync::Arc;

/// A plain action.
pub(in crate::people) type Act = Arc<dyn Fn() + Send + Sync>;
/// An action about something named by a key.
pub(in crate::people) type ActOn = Arc<dyn Fn(&str) + Send + Sync>;
/// The person typed text into the field with this key.
pub(in crate::people) type Typed = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// The things the view reports.
#[derive(Clone)]
pub(in crate::people) struct PeopleActions {
    /// A row was chosen.
    pub select: ActOn,
    /// Show more rows.
    pub more: Act,
    /// A new person.
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
    /// Keep the person after being asked.
    pub keep: Act,
    /// Add a field to this group.
    pub add_field: ActOn,
    /// Text typed.
    pub typed: Typed,
}
