//! Time for message lifetimes.
//!
//! Pattern: Strategy — the frontend asks a [`Clock`] and never the system, so lifetimes are testable.
//! [`SystemClock`] is the real one, [`ManualClock`] is set by hand (tests, pictures).

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Milliseconds on a clock that only goes forward.
pub trait Clock: Send + Sync {
    /// Now, in milliseconds since an arbitrary start.
    fn now_ms(&self) -> u64;
}

/// The system's monotonic clock.
#[derive(Debug)]
pub struct SystemClock(Instant);

impl SystemClock {
    /// Starts now.
    pub fn new() -> Self {
        Self(Instant::now())
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        u64::try_from(self.0.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
}

/// A clock that moves only when told to.
#[derive(Debug, Default)]
pub struct ManualClock(AtomicU64);

impl ManualClock {
    /// Move forward by `ms`.
    pub fn advance(&self, ms: u64) {
        self.0.fetch_add(ms, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now_ms(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}
