//! The guard of an answer: nothing a person did a moment ago, or aimed at something else, may
//! answer a question.
//!
//! Pattern: State + Strategy (`Clock`, injectable). A question that is newly shown, or has
//! changed (a new revision or digest), keeps its answer buttons disarmed for a time. A press only
//! counts if it began when the button was armed, the pointer did not leave before the release,
//! and the question did not change in between. Once one place has answered, the others for the
//! same call stay disarmed until the call changes. Nothing is decided here: it only says whether
//! a click may be reported.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Instant;

use super::model::ToolCall;

/// How long a new or changed question stays disarmed (provisional).
pub const ARM_DELAY_MS: u64 = 700;

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

type Key = (String, String);

struct Shown {
    revision: u64,
    digest: Vec<u8>,
    at: u64,
}

struct Press {
    button: String,
    revision: u64,
    digest: Vec<u8>,
}

struct Inner {
    clock: Arc<dyn Clock>,
    delay: u64,
    shown: HashMap<Key, Shown>,
    press: Option<Press>,
    answered: HashSet<Key>,
    refusals: HashMap<Key, String>,
}

/// The state shared by every place that can answer the same questions (a message card, the dock).
#[derive(Clone)]
pub struct Approvals(Arc<Mutex<Inner>>);

impl Default for Approvals {
    fn default() -> Self {
        Self::with_clock(Arc::new(SystemClock::default()))
    }
}

fn key(message: &str, call: &str) -> Key {
    (message.to_owned(), call.to_owned())
}

impl Approvals {
    /// With the real clock and the standard delay.
    pub fn new() -> Self {
        Self::default()
    }

    /// With this clock.
    pub fn with_clock(clock: Arc<dyn Clock>) -> Self {
        Self(Arc::new(Mutex::new(Inner {
            clock,
            delay: ARM_DELAY_MS,
            shown: HashMap::new(),
            press: None,
            answered: HashSet::new(),
            refusals: HashMap::new(),
        })))
    }

    /// Change the delay (milliseconds).
    #[must_use]
    pub fn delay_ms(self, ms: u64) -> Self {
        self.lock().delay = ms;
        self
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The delay in milliseconds.
    pub fn delay(&self) -> u64 {
        self.lock().delay
    }

    /// A card for `call` is drawn now. A new or changed question is disarmed from this moment.
    pub fn show(&self, message: &str, call: &ToolCall) {
        let mut g = self.lock();
        let k = key(message, &call.id);
        let now = g.clock.now_ms();
        let changed = g
            .shown
            .get(&k)
            .is_none_or(|s| s.revision != call.revision || s.digest != call.digest);
        if changed {
            g.shown.insert(
                k.clone(),
                Shown {
                    revision: call.revision,
                    digest: call.digest.clone(),
                    at: now,
                },
            );
            g.answered.remove(&k);
        }
    }

    /// The call is settled or gone: forget it.
    pub fn forget(&self, message: &str, call: &str) {
        let mut g = self.lock();
        let k = key(message, call);
        g.shown.remove(&k);
        g.answered.remove(&k);
        g.refusals.remove(&k);
    }

    /// May a press on this call's buttons begin an answer now?
    pub fn armed(&self, message: &str, call: &str) -> bool {
        let g = self.lock();
        let k = key(message, call);
        !g.answered.contains(&k)
            && g.shown
                .get(&k)
                .is_some_and(|s| g.clock.now_ms().saturating_sub(s.at) >= g.delay)
    }

    /// The mouse went down on `button` of the drawn state (`revision`, `digest`). True if it counts.
    pub fn press(
        &self,
        message: &str,
        call: &str,
        button: &str,
        revision: u64,
        digest: &[u8],
    ) -> bool {
        let ok = self.armed(message, call) && self.is_current(message, call, revision, digest);
        self.lock().press = ok.then(|| Press {
            button: button.to_owned(),
            revision,
            digest: digest.to_vec(),
        });
        ok
    }

    fn is_current(&self, message: &str, call: &str, revision: u64, digest: &[u8]) -> bool {
        self.lock()
            .shown
            .get(&key(message, call))
            .is_some_and(|s| s.revision == revision && s.digest == digest)
    }

    /// Is a press under way?
    pub fn has_press(&self) -> bool {
        self.lock().press.is_some()
    }

    /// The pointer left the button (or anything else cancelled the press).
    pub fn cancel_press(&self) {
        self.lock().press = None;
    }

    /// The mouse went up on `button`. True once, if the press began there, on this drawn state, and
    /// the question is still the one that was drawn; the call is then answered.
    pub fn release(
        &self,
        message: &str,
        call: &str,
        button: &str,
        revision: u64,
        digest: &[u8],
    ) -> bool {
        let began =
            self.lock().press.take().is_some_and(|p| {
                p.button == button && p.revision == revision && p.digest == digest
            });
        if !(began && self.armed(message, call) && self.is_current(message, call, revision, digest))
        {
            return false;
        }
        true
    }

    /// The answer was reported: this call is answered, and every other place for it stays disarmed
    /// until the call changes (the first answer wins).
    pub fn commit(&self, message: &str, call: &str) {
        let mut g = self.lock();
        g.answered.insert(key(message, call));
        g.refusals.remove(&key(message, call));
    }

    /// The engine refused the decision: show `reason` on the card of the (fresh) question.
    pub fn refused(&self, message: &str, call: &str, reason: &str) {
        self.lock()
            .refusals
            .insert(key(message, call), reason.to_owned());
    }

    /// The reason shown on this card, if the engine refused a decision.
    pub fn refusal(&self, message: &str, call: &str) -> Option<String> {
        self.lock().refusals.get(&key(message, call)).cloned()
    }
}

/// Draw again in `ms` milliseconds (a disarmed question becomes answerable at that moment).
pub(crate) fn refresh_later(live: crate::live::Live, ms: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(ms + 40));
        live.refresh();
    });
}
