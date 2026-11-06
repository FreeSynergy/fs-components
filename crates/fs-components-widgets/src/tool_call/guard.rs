//! The guard of an answer: nothing a person did a moment ago, or aimed at something else, may
//! answer a question.
//!
//! Pattern: State + Strategy (`Clock`, injectable). Every drawn question has an **entry** in the
//! place that draws it (a message card, the dock). An entry is new, and so disarmed for a time,
//! whenever the question it stands for is new or changed: another revision, digest, command or
//! summary, another position in the list, or having been away (an entry that is not drawn again
//! is forgotten, so a question that comes back is new). A press only counts if it began on an
//! armed button and the entry is the same one at the release (`generation`). An answered call
//! never arms again for the same revision, in any place. **Refuse is never delayed** (it cannot
//! be starved by a question that keeps changing); only Allow waits, needs a digest (an empty one
//! never matches) and, when the command does not fit its window, needs the person to have scrolled
//! it to the end. Nothing is decided here: it only says whether a click may be reported.

use blinc_layout::selector::ScrollRef;
use blinc_layout::widgets::scroll::ScrollPhysics;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, PoisonError};

use super::entry::{Body, Call, Entry, EntryKey, Press, call_key, entry_key, text_of};
use super::model::ToolCall;

use super::clock::{ARM_DELAY_MS, Clock, MIN_DELAY_MS, SystemClock};

struct Inner {
    clock: Arc<dyn Clock>,
    delay: u64,
    next: u64,
    entries: HashMap<EntryKey, Entry>,
    seen: HashSet<EntryKey>,
    press: Option<Press>,
    answered: HashMap<Call, u64>,
    refusals: HashMap<Call, String>,
}

/// The state shared by every place that can answer the same questions (a message card, the dock).
#[derive(Clone)]
pub struct Approvals(Arc<Mutex<Inner>>);

impl Default for Approvals {
    fn default() -> Self {
        Self::with_clock(Arc::new(SystemClock::default()))
    }
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
            next: 1,
            entries: HashMap::new(),
            seen: HashSet::new(),
            press: None,
            answered: HashMap::new(),
            refusals: HashMap::new(),
        })))
    }

    /// Change the delay (milliseconds; never below [`MIN_DELAY_MS`]).
    #[must_use]
    pub fn delay_ms(self, ms: u64) -> Self {
        self.lock().delay = ms.max(MIN_DELAY_MS);
        self
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The delay in milliseconds.
    pub fn delay(&self) -> u64 {
        self.lock().delay
    }

    /// A place starts drawing its questions.
    pub fn begin_place(&self, place: &str) {
        self.lock().seen.retain(|(p, ..)| p != place);
    }

    /// A place has drawn its questions: those it did not draw this time are forgotten, so one that
    /// comes back is a new one.
    pub fn end_place(&self, place: &str) {
        let mut g = self.lock();
        let seen = std::mem::take(&mut g.seen);
        g.entries.retain(|k, _| k.0 != place || seen.contains(k));
        g.seen = seen;
    }

    /// A card for `call` is drawn in `place` at position `slot`. `needs_read`: its command does not
    /// fit its window, so it must be scrolled to the end before Allow works.
    pub(crate) fn show(
        &self,
        place: &str,
        message: &str,
        call: &ToolCall,
        slot: usize,
        needs_read: bool,
    ) -> Body {
        let mut g = self.lock();
        let k = entry_key(place, message, &call.id);
        g.seen.insert(k.clone());
        let text = text_of(call);
        let same = g.entries.get(&k).is_some_and(|e| {
            e.revision == call.revision
                && e.digest == call.digest
                && e.text == text
                && e.slot == slot
                && e.needs_read == needs_read
        });
        if !same {
            let (generation, at) = (g.next, g.clock.now_ms());
            g.next += 1;
            let body = Body {
                scroll: ScrollRef::new(),
                physics: Arc::new(Mutex::new(ScrollPhysics::default())),
            };
            g.entries.insert(
                k.clone(),
                Entry {
                    generation,
                    revision: call.revision,
                    digest: call.digest.clone(),
                    text,
                    slot,
                    at,
                    needs_read,
                    body,
                    cursor: 0,
                },
            );
        }
        g.entries[&k].body.clone()
    }

    /// The option under the keyboard cursor of this question (the first one to begin with).
    pub fn cursor(&self, place: &str, message: &str, call: &str) -> usize {
        self.lock()
            .entries
            .get(&entry_key(place, message, call))
            .map_or(0, |e| e.cursor)
    }

    /// Move the keyboard cursor over `len` options.
    pub fn move_cursor(&self, place: &str, message: &str, call: &str, delta: i32, len: usize) {
        if let Some(e) = self
            .lock()
            .entries
            .get_mut(&entry_key(place, message, call))
        {
            let at = i64::try_from(e.cursor).unwrap_or(0) + i64::from(delta);
            e.cursor =
                usize::try_from(at.clamp(0, i64::try_from(len).unwrap_or(1) - 1)).unwrap_or(0);
        }
    }

    /// The call is settled or gone: forget it everywhere.
    pub fn forget(&self, message: &str, call: &str) {
        let mut g = self.lock();
        g.entries.retain(|k, _| !(k.1 == message && k.2 == call));
        g.answered.remove(&call_key(message, call));
        g.refusals.remove(&call_key(message, call));
    }

    /// May a press on this call's button begin an answer now? `refuse` buttons are never delayed.
    pub fn armed(&self, place: &str, message: &str, call: &str, refuse: bool) -> bool {
        let g = self.lock();
        let Some(e) = g.entries.get(&entry_key(place, message, call)) else {
            return false;
        };
        if g.answered.get(&call_key(message, call)) == Some(&e.revision) {
            return false;
        }
        if refuse {
            return true;
        }
        let waited = g.clock.now_ms().saturating_sub(e.at) >= g.delay;
        // in doubt (no size reported yet): not read
        let read = !e.needs_read
            || (e.body.scroll.content_size().is_some() && e.body.scroll.is_at_bottom());
        waited && read && !e.digest.is_empty()
    }

    /// The mouse (or the key) went down on `button`. True if it counts.
    pub fn press(
        &self,
        place: &str,
        message: &str,
        call: &str,
        button: &str,
        refuse: bool,
    ) -> bool {
        let ok = self.armed(place, message, call, refuse);
        let mut g = self.lock();
        let generation = g
            .entries
            .get(&entry_key(place, message, call))
            .map(|e| e.generation);
        g.press = generation.filter(|_| ok).map(|generation| Press {
            button: button.to_owned(),
            generation,
        });
        ok
    }

    /// Is a press under way?
    pub fn has_press(&self) -> bool {
        self.lock().press.is_some()
    }

    /// The pointer left the button (or anything else cancelled the press).
    pub fn cancel_press(&self) {
        self.lock().press = None;
    }

    /// The mouse (or the key) went up on `button`. True once, if the press began on this button, the
    /// entry is still the one it began on, and the button is still armed.
    pub fn release(
        &self,
        place: &str,
        message: &str,
        call: &str,
        button: &str,
        refuse: bool,
    ) -> bool {
        let press = self.lock().press.take();
        let generation = self
            .lock()
            .entries
            .get(&entry_key(place, message, call))
            .map(|e| e.generation);
        press.is_some_and(|p| p.button == button && Some(p.generation) == generation)
            && self.armed(place, message, call, refuse)
    }

    /// The answer was reported: this call is answered for this revision, in every place, until it
    /// changes.
    pub fn commit(&self, message: &str, call: &str, revision: u64) {
        let mut g = self.lock();
        g.answered.insert(call_key(message, call), revision);
        g.refusals.remove(&call_key(message, call));
    }

    /// The engine refused the decision: show `reason`, lift the answered mark and disarm the call
    /// again for the delay, so the person can decide anew (also when nothing changed).
    pub fn refused(&self, message: &str, call: &str, reason: &str) {
        let mut g = self.lock();
        g.answered.remove(&call_key(message, call));
        g.refusals
            .insert(call_key(message, call), reason.to_owned());
        let (now, mut next) = (g.clock.now_ms(), g.next);
        for (k, e) in g
            .entries
            .iter_mut()
            .filter(|(k, _)| k.1 == message && k.2 == call)
        {
            let _ = k;
            e.at = now;
            e.generation = next;
            next += 1;
        }
        g.next = next;
    }

    /// The reason shown on this card, if the engine refused a decision.
    pub fn refusal(&self, message: &str, call: &str) -> Option<String> {
        self.lock().refusals.get(&call_key(message, call)).cloned()
    }
}

/// Draw again in `ms` milliseconds (a disarmed question becomes answerable at that moment).
pub(crate) fn refresh_later(live: crate::live::Live, ms: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(ms + 40));
        live.refresh_now();
    });
}
