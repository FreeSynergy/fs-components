//! What a plan widget tells its host, and how it asks for a new drawing.
//!
//! Pattern: Observer — plain callbacks the host gives; after one has run the widget asks the window to draw again
//! (the host's state decides what the new drawing holds). Hides how Blinc is asked to rebuild.

use std::sync::Arc;

use blinc_core::BlincContextState;

/// A callback with one text: a key, or what was typed.
pub type Say = Arc<dyn Fn(&str) + Send + Sync>;

/// A callback with two texts: a column and a key, or a column and what was typed.
pub type Say2 = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// The callbacks of the plan screens. Every one is optional; a missing one does nothing.
#[derive(Clone, Default)]
pub struct Events {
    /// A view tab was chosen (its key).
    pub tab: Option<Say>,
    /// A card, a row or a list of the side was chosen (its key).
    pub pick: Option<Say>,
    /// The box beside a task was ticked (its key).
    pub tick: Option<Say>,
    /// A row asks to open or close its details (its key).
    pub open: Option<Say>,
    /// "+ Create" in a column was pressed (the column's key).
    pub create: Option<Say>,
    /// A title was entered: the place (column key or list key) and the text.
    pub enter: Option<Say2>,
    /// The panel was closed.
    pub close: Option<Arc<dyn Fn() + Send + Sync>>,
}

/// Ask the window to draw again. Before the window is up there is nothing to draw.
pub fn redraw() {
    if BlincContextState::is_initialized() {
        BlincContextState::get().request_rebuild();
    }
}

/// Run `f` of `events` with `key`, then draw again.
pub(super) fn tell(f: &Option<Say>, key: &str) {
    if let Some(f) = f {
        f(key);
        redraw();
    }
}
