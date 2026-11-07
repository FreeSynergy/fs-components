//! Presses and answers: what began an answer, and when it is reported.
//!
//! Pattern: State (a press is begun on one armed button of one entry, and counts only if it ends
//! on that button while the entry is the same one). Nothing is decided here beyond "may this be
//! reported".

use super::{Approvals, call_key, entry_key};

impl Approvals {
    /// Enter or Space went down on the answer row of this question. True for the first press of
    /// the key only: its repeats, also after a redraw, come while it is held and are no new press.
    pub fn key_down(&self, place: &str, message: &str, call: &str) -> bool {
        self.lock().held.insert(entry_key(place, message, call))
    }

    /// Enter or Space went up (or the row lost the keyboard). True if it was held.
    pub fn key_up(&self, place: &str, message: &str, call: &str) -> bool {
        self.lock().held.remove(&entry_key(place, message, call))
    }

    /// Is Enter or Space held on the answer row of this question? A click that arrives meanwhile
    /// is a repeat of that key.
    pub fn is_held(&self, place: &str, message: &str, call: &str) -> bool {
        self.lock().held.contains(&entry_key(place, message, call))
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
