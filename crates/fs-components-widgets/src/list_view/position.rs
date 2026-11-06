//! Where the list is scrolled to.
//!
//! Pattern: Memento — a handle the owner keeps, so that the scroll position and the row under the
//! keyboard survive a rebuild of the list. Nothing here hands out a lock.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::selector::ScrollRef;
use blinc_layout::widgets::scroll::{ScrollPhysics, SharedScrollPhysics};

/// No row: the value of the atomics below.
const NONE: usize = usize::MAX;

/// The scroll position of one list. Cheap to clone; clones share the position.
#[derive(Clone)]
pub struct ListPosition {
    pub(super) scroll: ScrollRef,
    pub(super) physics: SharedScrollPhysics,
    wanted: Arc<AtomicUsize>,
    cursor: Arc<AtomicUsize>,
}

impl Default for ListPosition {
    fn default() -> Self {
        Self {
            scroll: ScrollRef::new(),
            physics: Arc::new(Mutex::new(ScrollPhysics::default())),
            wanted: Arc::new(AtomicUsize::new(NONE)),
            cursor: Arc::new(AtomicUsize::new(NONE)),
        }
    }
}

impl ListPosition {
    /// At the top.
    pub fn new() -> Self {
        Self::default()
    }

    /// Show this row at the top of the list the next time it is built.
    pub fn scroll_to_row(&self, row: usize) {
        self.wanted.store(row.min(NONE - 1), Ordering::Relaxed);
    }

    /// The row asked for with [`ListPosition::scroll_to_row`], once.
    pub(super) fn take_wanted(&self) -> Option<usize> {
        match self.wanted.swap(NONE, Ordering::Relaxed) {
            NONE => None,
            row => Some(row),
        }
    }

    /// How far the list is scrolled now, in pixels from the top.
    pub fn offset(&self) -> f32 {
        -self.lock_offset()
    }

    /// The row under the keyboard, if any.
    pub fn cursor(&self) -> Option<usize> {
        match self.cursor.load(Ordering::Relaxed) {
            NONE => None,
            row => Some(row),
        }
    }

    /// Set the row under the keyboard.
    pub fn set_cursor(&self, row: Option<usize>) {
        self.cursor.store(row.unwrap_or(NONE), Ordering::Relaxed);
    }

    fn lock_offset(&self) -> f32 {
        self.physics
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .offset_y
    }

    /// Put the list at `offset` pixels from the top, also in the element already on screen.
    pub(super) fn apply_offset(&self, offset: f32) {
        self.physics
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .offset_y = -offset;
        self.scroll.set_scroll_offset(0.0, offset);
    }
}
