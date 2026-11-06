//! `sortable-list`: reorder items within a list and between lists, also by keyboard.
//!
//! Pattern: State + Command + Adapter. The move logic is a pure state machine ([`Sorter`]: lanes,
//! a cursor, an item held) that knows no Blinc; a [`Command`] is one step of it, and
//! [`Bindings`] (optional) turn keys into commands. The frontend feeds gestures in through the
//! same methods (`grab`, `hover`, `drop_grabbed`, `cancel`), ADR 0037. [`SortableList`] is the
//! shared handle plus the Blinc view of the state; the caller hears of every step as an
//! [`Outcome`] and keeps its own data: a move that it does not accept is undone by `set_lanes`.

mod bindings;
mod handle;
mod model;
mod moves;
mod state;
mod steps;
mod view;
mod window;

pub use bindings::{Bindings, Command};
pub use handle::{Labels, SortableList};
pub use model::{Item, Lane, Moved, Outcome, Place, Pos, Refusal};
pub use state::Sorter;
pub use window::window;

/// CSS of the sortable list: only theme variables.
pub const STYLES: &str = r"
.fs-sl { display: flex; flex-direction: row; gap: var(--space-3); align-items: flex-start; width: 100%; }
.fs-sl-lane { display: flex; flex-direction: column; flex-grow: 1; flex-basis: 0; min-width: 0; gap: var(--space-1); padding: var(--space-2); border-radius: var(--radius-default); background: var(--surface); border-width: 1px; border-color: var(--border); }
.fs-sl-title { color: var(--text-secondary); font-size: var(--text-sm); padding: var(--space-1); }
.fs-sl-row { display: flex; flex-direction: row; align-items: center; min-width: 0; overflow: hidden; padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); background: var(--surface-elevated); border-width: 2px; border-color: var(--border); color: var(--text-primary); }
.fs-sl-row-cursor { border-color: var(--border-focus); }
.fs-sl-row-grabbed { border-color: var(--primary); background: var(--accent-subtle); }
.fs-sl-row-locked { color: var(--text-tertiary); }
.fs-sl-note { color: var(--text-secondary); font-size: var(--text-xs); padding: var(--space-1) var(--space-2); }
";
