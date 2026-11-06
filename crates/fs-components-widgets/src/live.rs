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
/// The least time between two draws of one widget caused by `refresh` (milliseconds); change it
/// with [`set_min_gap_ms`].
pub const MIN_GAP_MS: u64 = 60;
static GAP: AtomicU64 = AtomicU64::new(u64::MAX);

/// Cap the gap between two draws caused by `refresh` for every widget (0: every refresh draws, as
/// tests want).
pub fn set_min_gap_ms(ms: u64) {
    GAP.store(ms, Ordering::Relaxed);
}
/// Per widget: the time of its last draw and whether a trailing draw waits.
static THROTTLE: std::sync::Mutex<std::collections::BTreeMap<u64, (u64, bool)>> =
    std::sync::Mutex::new(std::collections::BTreeMap::new());

fn now_ms() -> u64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    u64::try_from(
        START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_millis(),
    )
    .unwrap_or(u64::MAX)
}

/// The redraw handle of one widget instance (also for workbenches that join several widgets). Cheap to copy.
#[derive(Clone, Copy, Debug)]
pub struct Live {
    id: u64,
    gap: u64,
}

impl Live {
    /// The least time between two draws of this widget caused by `refresh`. A widget whose draw is
    /// costly sets a long one: a draw that takes longer than the gap between two feeds would keep the
    /// interface thread busy for good (found with a feed every 40 ms: the window froze).
    #[must_use]
    pub fn gap(mut self, ms: u64) -> Self {
        self.gap = ms;
        self
    }

    /// A new instance with its own counter.
    pub fn new() -> Self {
        Self {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
            gap: 0,
        }
    }

    /// A signal, not a `State`: `State::update` does not reach a live area when the notifier of the
    /// window was not installed (found by test), a signal always does.
    fn counter(self) -> Signal<u64> {
        BlincContextState::get().use_signal_keyed(&format!("fs-live-{}", self.id), || 0u64)
    }

    /// Ask the view to draw again soon: at most one draw per [`MIN_GAP_MS`] however fast the host
    /// feeds (a draw that costs more than the gap between two feeds would starve every frame), the
    /// last change always gets its draw. Nothing happens when no window or picture exists yet.
    pub fn refresh(self) {
        if !BlincContextState::is_initialized() {
            return;
        }
        let now = now_ms();
        let spawn = {
            let mut map = THROTTLE
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let slot = map.entry(self.id).or_default();
            if now.saturating_sub(slot.0) >= self.gap.min(GAP.load(Ordering::Relaxed)) {
                None
            } else if slot.1 {
                Some(false) // a trailing draw is already waiting
            } else {
                slot.1 = true;
                Some(true)
            }
        };
        match spawn {
            None => self.refresh_now(),
            Some(true) => {
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(
                        self.gap.min(GAP.load(Ordering::Relaxed)),
                    ));
                    self.refresh_now();
                });
            }
            Some(false) => {}
        }
    }

    /// Draw again at once (a question appearing must not wait).
    pub fn refresh_now(self) {
        if BlincContextState::is_initialized() {
            {
                let mut map = THROTTLE
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                *map.entry(self.id).or_default() = (now_ms(), false);
            }
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
