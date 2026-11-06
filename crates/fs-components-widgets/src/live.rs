//! A widget that draws again when its state changes.
//!
//! Pattern: Observer — a counter signal per widget instance; the view depends on it and a change
//! of the state bumps it. Hides how Blinc keeps state across rebuilds.

use std::sync::atomic::{AtomicU64, Ordering};

use blinc_core::BlincContextState;
use blinc_core::reactive::Signal;
use blinc_layout::div::Div;
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::stateful_with_key;

static NEXT: AtomicU64 = AtomicU64::new(1);
static TICK: AtomicU64 = AtomicU64::new(1);

/// The redraw handle of one widget instance (also for workbenches that join several widgets). Cheap to copy.
#[derive(Clone, Copy, Debug)]
pub struct Live {
    id: u64,
}

impl Live {
    /// A new instance with its own counter.
    pub fn new() -> Self {
        Self {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
        }
    }

    /// A signal, not a `State`: `State::update` does not reach a live area when the notifier of the
    /// window was not installed (found by test), a signal always does.
    fn counter(self) -> Signal<u64> {
        BlincContextState::get().use_signal_keyed(&format!("fs-live-{}", self.id), || 0u64)
    }

    /// Ask the view to draw again (nothing to do when no window or picture exists yet).
    pub fn refresh(self) {
        if BlincContextState::is_initialized() {
            // a fresh number every time: two refreshes before one frame must not write the same value
            self.counter().set(TICK.fetch_add(1, Ordering::Relaxed));
        }
    }

    /// A container that builds `view` again after every [`Live::refresh`]. `slot` names the area:
    /// two areas of one widget must have two names (one name makes them one area).
    pub fn region<E, F>(self, slot: &str, view: F) -> impl crate::Element + use<E, F>
    where
        E: crate::Element + 'static,
        F: Fn() -> E + Send + Sync + 'static,
    {
        self.view(slot, move || {
            blinc_layout::div::div().w_full().child(view())
        })
    }

    pub(crate) fn view(self, slot: &str, view: impl Fn() -> Div + Send + Sync + 'static) -> Div {
        let live = stateful_with_key::<NoState>(format!("fs-live-view-{}-{slot}", self.id))
            .deps([self.counter().id()])
            .on_state(move |_| view());
        blinc_layout::div::div().w_full().child(live)
    }
}

impl Default for Live {
    fn default() -> Self {
        Self::new()
    }
}
