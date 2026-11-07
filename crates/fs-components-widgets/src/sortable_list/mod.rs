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
mod styles;
mod view;
mod window;

pub use bindings::{Bindings, Command};
pub use handle::{Labels, SortableList};
pub use model::{Chosen, Item, Lane, Moved, Outcome, Place, Pos, Refusal, Source};
pub use state::Sorter;
pub use styles::STYLES;
pub use view::Content;
pub use window::window;
