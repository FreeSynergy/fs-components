//! What the person can do, as callbacks.
//!
//! Pattern: none of the table fits (plain closures, no listener trait) — the widget decides what each one does: change the state
//! and draw again.

use std::sync::Arc;

use fs_components_widgets::calendar_view::OnPick;

/// A plain action.
pub(in crate::calendar) type Act = Arc<dyn Fn() + Send + Sync>;
/// An action about something named by a key.
pub(in crate::calendar) type ActOn = Arc<dyn Fn(&str) + Send + Sync>;
/// The person typed text into the field with this key.
pub(in crate::calendar) type Typed = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// The things the view reports.
#[derive(Clone)]
pub(in crate::calendar) struct CalendarActions {
    /// Go to today.
    pub today: Act,
    /// One step back.
    pub previous: Act,
    /// One step on.
    pub next: Act,
    /// Show a month.
    pub month: Act,
    /// Show a week.
    pub week: Act,
    /// Show a day.
    pub day: Act,
    /// Show the agenda.
    pub agenda: Act,
    /// A click on the calendar.
    pub pick: OnPick,
    /// A new item.
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
    /// Back to the calendar.
    pub back: Act,
    /// The first press of Delete: asks.
    pub ask_delete: Act,
    /// Really delete.
    pub confirm_delete: Act,
    /// Keep the item after being asked.
    pub keep: Act,
    /// A yes-or-no field was pressed.
    pub toggle: ActOn,
    /// Text typed.
    pub typed: Typed,
}
