//! The time the guard goes by.
//!
//! Pattern: Strategy (`Clock`, injectable, so tests need no waiting).

use std::time::Instant;

/// How long a new or changed question keeps Allow disarmed (provisional).
pub const ARM_DELAY_MS: u64 = 700;
/// The least delay that can be asked for: arming cannot be switched off.
pub const MIN_DELAY_MS: u64 = 300;

/// The time the guard goes by. Injectable, so tests need no waiting.
pub trait Clock: Send + Sync {
    /// Milliseconds since some fixed start.
    fn now_ms(&self) -> u64;
}

/// The real clock.
pub struct SystemClock(Instant);

impl Default for SystemClock {
    fn default() -> Self {
        Self(Instant::now())
    }
}

impl Clock for SystemClock {
    #[allow(clippy::cast_possible_truncation)]
    fn now_ms(&self) -> u64 {
        self.0.elapsed().as_millis() as u64
    }
}
