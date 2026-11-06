//! The calendar workbench as a thing a program can place in its own window.
//!
//! Pattern: Facade + Observer — `CalendarWidget` holds the shared state and builds the element. Every
//! change asks the window for a full rebuild: there is no live area and no `stateful` here, so nothing
//! decided in a build can go stale and no old callback keeps running.
//!
//! No lock is held while elements are built: views draw from a snapshot, and the state is reached only
//! through a closure.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_core::BlincContextState;
use fs_components_widgets::{CalendarPosition, Element};

use super::labels::CalendarLabels;
use super::shown::Variant;
use super::source::CalendarSource;
use super::state::CalendarState;
use super::view::{InputSlots, Places, workbench};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// The calendar workbench, ready to be built into a layout. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct CalendarWidget {
    id: u64,
    pub(super) state: Arc<Mutex<CalendarState>>,
    position: CalendarPosition,
    slots: InputSlots,
}

impl CalendarWidget {
    /// The whole workbench over `source`.
    pub fn new(source: Arc<dyn CalendarSource>, labels: CalendarLabels) -> Self {
        Self::with_variant(source, labels, Variant::Full)
    }

    /// The compact variant (ADR 0037): the agenda, navigation and reading, whatever the source can do.
    pub fn compact(source: Arc<dyn CalendarSource>, labels: CalendarLabels) -> Self {
        Self::with_variant(source, labels, Variant::Compact)
    }

    fn with_variant(
        source: Arc<dyn CalendarSource>,
        labels: CalendarLabels,
        variant: Variant,
    ) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            state: Arc::new(Mutex::new(CalendarState::new(source, labels, variant))),
            position: CalendarPosition::new(),
            slots: InputSlots::default(),
        }
    }

    /// Run `f` with the state, for a program's menus and commands. A closure, so the lock cannot be
    /// held across calls. After a change made this way call [`CalendarWidget::refresh`]. Do not call
    /// other methods of this widget from inside `f`.
    pub fn with_state<T>(&self, f: impl FnOnce(&mut CalendarState) -> T) -> T {
        f(&mut locked(&self.state))
    }

    /// The style sheet to add to the theme of the window.
    pub fn styles(&self) -> String {
        super::view::styles()
    }

    /// Draw again after the state was changed through [`CalendarWidget::with_state`].
    pub fn refresh(&self) {
        rebuild();
    }

    /// The element of the workbench, filling `width` x `height`. Call it inside the program's build
    /// function, not inside a live area.
    pub fn build(&self, width: f32, height: f32) -> impl Element + use<> {
        let snapshot = locked(&self.state).clone();
        let places = Places {
            name: self.element_name(),
            position: self.position.clone(),
            slots: self.slots.clone(),
        };
        workbench(&snapshot, &places, &self.actions(), width, height)
    }

    /// The name element ids of this widget begin with: `<name>-save`, `<name>-today`, ... and the ids
    /// of the calendar view `<name>-day-<date>`, `<name>-item-<key>@<date>`.
    pub fn element_name(&self) -> String {
        format!("fs-calendar-{}", self.id)
    }
}

pub(super) fn locked(state: &Mutex<CalendarState>) -> MutexGuard<'_, CalendarState> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Ask the window to build everything again. Before the runtime is up there is nothing to draw.
pub(super) fn rebuild() {
    if BlincContextState::is_initialized() {
        BlincContextState::get().request_rebuild();
    }
}
