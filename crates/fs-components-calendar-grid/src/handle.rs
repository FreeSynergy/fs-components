//! The handle of a calendar grid: shared state, data source, events, and the redraw.
//!
//! Pattern: Facade + Observer — one clonable handle over [`GridState`]; each call changes the
//! state, tells the host through [`GridEvents`], and asks the live area to draw again (only the
//! grid is built again, not the window). Every gesture, button and key goes through these methods.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use fs_components_fields::{Look, Say};
use fs_components_widgets::Key;
use fs_components_widgets::live::Live;

use crate::model::{Mode, Nav, Slot};
use crate::source::{GridEvents, GridSource, NoEvents};
use crate::state::{GridState, KeyOutcome};

/// A calendar grid. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct CalendarGrid {
    pub(crate) state: Arc<Mutex<GridState>>,
    pub(crate) source: Arc<dyn GridSource>,
    events: Arc<dyn GridEvents>,
    pub(crate) say: Arc<dyn Say>,
    pub(crate) look: Look,
    pub(crate) live: Live,
}

impl CalendarGrid {
    /// A grid over `state`, with items from `source`, words from `say`, in `look`.
    pub fn new(
        state: GridState,
        source: Arc<dyn GridSource>,
        say: Arc<dyn Say>,
        look: Look,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
            source,
            events: Arc::new(NoEvents),
            say,
            look,
            live: Live::new(),
        }
    }

    /// Tell `events` of the reader's choices.
    #[must_use]
    pub fn events(mut self, events: Arc<dyn GridEvents>) -> Self {
        self.events = events;
        self
    }

    /// A copy of the state.
    pub fn snapshot(&self) -> GridState {
        self.lock().clone()
    }

    pub(crate) fn lock(&self) -> MutexGuard<'_, GridState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Previous, next or today.
    pub fn navigate(&self, nav: Nav) {
        let anchor = {
            let mut s = self.lock();
            s.navigate(nav);
            s.anchor
        };
        self.events.navigated(nav, anchor);
        self.live.refresh();
    }

    /// Switch to month, week or day.
    pub fn set_mode(&self, mode: Mode) {
        self.lock().set_mode(mode);
        self.live.refresh();
    }

    /// The reader chose a slot.
    pub fn select_slot(&self, slot: Slot) {
        self.lock().select_slot(slot);
        self.events.select_slot(slot);
        self.live.refresh();
    }

    /// The reader chose an item.
    pub fn select_item(&self, key: &str) {
        self.lock().select_item(key);
        self.events.select_item(key);
        self.live.refresh();
    }

    /// A key press; true when the grid used it.
    pub fn key(&self, key: Key) -> bool {
        let (outcome, before, after) = {
            let mut s = self.lock();
            let before = (s.anchor, s.days()[0]);
            let outcome = s.key(key);
            (outcome, before, (s.anchor, s.days()[0]))
        };
        if let KeyOutcome::Chosen(slot) = outcome {
            self.events.select_slot(slot);
        }
        // Tell the host of a new range as the buttons do: Page keys and Home always, arrows when they
        // carried the cursor into another range.
        let paged = matches!(key, Key::PageUp | Key::PageDown | Key::Home);
        if after.0 != before.0 && (paged || after.1 != before.1) {
            let nav = match key {
                Key::Home => Nav::Today,
                _ if after.0 < before.0 => Nav::Previous,
                _ => Nav::Next,
            };
            self.events.navigated(nav, after.0);
        }
        if outcome != KeyOutcome::Ignored {
            self.live.refresh();
        }
        outcome != KeyOutcome::Ignored
    }
}

impl CalendarGrid {
    /// The element of `width` x `height` pixels. The keyboard binding sits on a stable outer box;
    /// inside it a live area is built again after each change, nothing else.
    pub fn build(
        &self,
        width: f32,
        height: f32,
    ) -> impl blinc_layout::prelude::ElementBuilder + use<> {
        let me = self.clone();
        let keys = self.clone();
        let inner = self
            .live
            .region("grid", move || crate::view::grid(&me, width, height));
        crate::view::bind_keys(
            blinc_layout::div::div().id("cal-root").w(width).h(height),
            Arc::new(move |k| keys.key(k)),
        )
        .child(inner)
    }
}
