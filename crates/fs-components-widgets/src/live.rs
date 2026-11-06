//! A widget that draws again when its state changes.
//!
//! Pattern: Observer — a counter signal per widget instance; the view depends on it and a change
//! of the state bumps it. Hides how Blinc keeps state across rebuilds.

use std::sync::atomic::{AtomicU64, Ordering};

use blinc_core::{BlincContextState, State};
use blinc_layout::div::Div;
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::stateful_with_key;

static NEXT: AtomicU64 = AtomicU64::new(1);

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

    fn counter(self) -> State<u64> {
        BlincContextState::get().use_state_keyed(&format!("fs-live-{}", self.id), || 0u64)
    }

    /// Ask the view to draw again (nothing to do when no window or picture exists yet).
    pub fn refresh(self) {
        if BlincContextState::is_initialized() {
            self.counter().update(|n| n + 1);
        }
    }

    /// A container that builds `view` again after every [`Live::refresh`].
    pub fn view(self, view: impl Fn() -> Div + Send + Sync + 'static) -> Div {
        let live = stateful_with_key::<NoState>(format!("fs-live-view-{}", self.id))
            .deps([self.counter().signal_id()])
            .on_state(move |_| view());
        blinc_layout::div::div().w_full().child(live)
    }
}

impl Default for Live {
    fn default() -> Self {
        Self::new()
    }
}
