//! When a disarmed question must be drawn again.
//!
//! Pattern: State (a deadline per question: asked for, or not). A question that is not armed yet
//! becomes armed by the clock alone, so it must be drawn once more at that moment, but only once:
//! every drawing asks, so the guard answers "yes" to the first and "no" until the moment has
//! passed. A question that time cannot arm (empty digest, a command not scrolled to its end,
//! answered) asks for nothing; scrolling a long command to its end (or away from it) asks once, at
//! that change, through a watch on the scroll offset.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use blinc_core::owner::Owner;
use blinc_core::reactive::effect;
use blinc_layout::selector::ScrollRef;

use super::{Approvals, Watch, entry_key};

/// Draw again in this many milliseconds.
pub(crate) type Rearm = Arc<dyn Fn(u64) + Send + Sync>;

/// Slack after the delay, so the redraw finds the question armed.
const SLACK_MS: u64 = 40;

impl Approvals {
    /// Milliseconds after which `call` should be drawn again, if it should be and no redraw is
    /// already asked for. Asking is recorded: until that moment has passed, the answer is `None`.
    pub(crate) fn refresh_due(&self, place: &str, message: &str, call: &str) -> Option<u64> {
        let mut g = self.lock();
        let k = entry_key(place, message, call);
        let now = g.clock.now_ms();
        let e = g.entries.get(&k)?;
        let left = (e.at + g.delay).saturating_sub(now);
        let read = !e.needs_read
            || (e.body.scroll.content_size().is_some() && e.body.scroll.is_at_bottom());
        let answered = g.answered.get(&super::call_key(message, call)) == Some(&e.revision);
        if left == 0 || e.digest.is_empty() || !read || answered {
            return None;
        }
        if g.pending.get(&k).is_some_and(|until| *until > now) {
            return None;
        }
        g.pending.insert(k, now + left + SLACK_MS);
        g.asked += 1;
        Some(left)
    }

    /// A command that does not fit its window arms Allow when it is scrolled to its end, and
    /// disarms it when it is scrolled back: draw once when the window gains or loses its end, and
    /// not while it only moves (the scroll reports its offset to a signal, this watches it). The
    /// watch is the entry's, owned by no build; the entry gives it back when it goes.
    pub(super) fn watch_end(&self, scroll: &ScrollRef, rearm: &Rearm) -> Watch {
        let (this, scroll, rearm) = (Arc::downgrade(&self.0), scroll.clone(), Arc::clone(rearm));
        Owner::detached(|| {
            let offset = scroll.watch_y(0.0);
            let at_end = AtomicBool::new(false);
            let run = effect(move |g| {
                let _ = g.get(offset);
                let now = scroll.content_size().is_some() && scroll.is_at_bottom();
                if at_end.swap(now, Ordering::Relaxed) != now {
                    if let Some(inner) = this.upgrade() {
                        Self(inner).lock().asked += 1;
                    }
                    rearm(0);
                }
            });
            Watch {
                effect: run.id(),
                offset: offset.id(),
            }
        })
    }

    /// How many redraws the guard has asked for so far.
    pub fn redraws_asked(&self) -> u64 {
        self.lock().asked
    }
}

/// Draw again in `ms` milliseconds (a disarmed question becomes answerable at that moment).
pub(crate) fn refresh_later(live: crate::live::Live, ms: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(ms + SLACK_MS));
        live.refresh_now();
    });
}
