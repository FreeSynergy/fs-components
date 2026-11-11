//! Moving about in the conversation: following, jumping, scrolling, older messages at the front.
//!
//! Pattern: Facade (more of the handle's methods, kept apart for size). The reader away from the
//! bottom is never moved by what arrives; at the bottom the list follows.

use super::handle::{Inner, MessageList};
use crate::live::Live;
use std::ops::Range;
use std::sync::{Arc, Mutex, PoisonError};

use super::window::covered;

impl MessageList {
    /// `n` older messages were inserted before the first one (the host loaded an earlier page). The
    /// reader keeps the row being read where it is; nothing is counted as new.
    pub fn refresh_prepended(&self, n: usize) {
        let shift = {
            let mut inner = self.lock();
            let before = inner.heights.total();
            inner.heights.prepend(n);
            let at = inner.revs.len();
            inner.revs.splice(0..0, std::iter::repeat_n(0, n));
            debug_assert_eq!(inner.revs.len(), at + n);
            inner.built = 0..0;
            inner.shown.clear();
            let count = inner.source.count();
            inner.heights.grow(count);
            inner.revs.resize(count, 0);
            if inner.follow {
                0.0
            } else {
                inner.heights.total() - before
            }
        };
        if shift > 0.0 {
            self.position.apply_offset(self.position.offset() + shift);
        }
        self.live.refresh();
    }

    /// Distance in pixels from the top of the conversation to the top of message `index`, as known now.
    pub fn top_of(&self, index: usize) -> f32 {
        self.lock().heights.top(index)
    }

    /// The message at `y` pixels from the top, as known now.
    pub fn index_at(&self, y: f32) -> usize {
        self.lock().heights.at(y)
    }

    /// Scroll to the newest message and follow it from now on.
    pub fn jump_to_end(&self) {
        {
            let mut inner = self.lock();
            inner.follow = true;
            inner.unseen = 0;
        }
        self.live.refresh();
    }

    /// Does the list follow the newest message (the reader is at the bottom)?
    pub fn is_following(&self) -> bool {
        self.lock().follow
    }

    /// Messages that arrived while the reader was away from the bottom.
    pub fn unseen(&self) -> usize {
        self.lock().unseen
    }

    /// Pixels scrolled from the top.
    pub fn offset(&self) -> f32 {
        self.position.offset()
    }

    /// Total height of the conversation as known now (estimates and measurements).
    pub fn total_height(&self) -> f32 {
        self.lock().heights.total()
    }

    /// The messages built now (indexes).
    pub fn built(&self) -> Range<usize> {
        self.lock().built.clone()
    }

    /// Scroll by `pixels` (positive = towards newer messages), as a wheel would.
    pub fn scroll_by(&self, pixels: f32) {
        let y = (self.position.offset() + pixels).max(0.0);
        self.position.apply_offset(y);
        self.on_scroll(y);
    }

    /// The scroll position changed (called from the scroll watcher; also by tests).
    pub fn on_scroll(&self, y: f32) {
        Self::scrolled(&self.shared, self.live, y);
    }

    /// What [`MessageList::on_scroll`] does, from the parts the scroll watcher keeps.
    pub(super) fn scrolled(shared: &Arc<Mutex<Inner>>, live: Live, y: f32) {
        let rebuild = {
            let mut inner = shared.lock().unwrap_or_else(PoisonError::into_inner);
            if inner.built.is_empty() {
                return; // nothing drawn yet: the first drawing decides where the list stands
            }
            let view = inner.size.1;
            let at_end = y + view + 4.0 >= inner.heights.total();
            if at_end {
                inner.unseen = 0;
            }
            let changed = inner.follow != at_end;
            inner.follow = at_end;
            let built = inner.built.clone();
            changed || !covered(&inner.heights, &built, y, view)
        };
        if rebuild {
            // Called from inside the reactive graph (the scroll watcher): drawing again from here
            // would write a signal while the graph is locked, so it is done from outside.
            std::thread::spawn(move || live.refresh());
        }
    }
}
