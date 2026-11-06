//! Where the list is scrolled to.
//!
//! Pattern: Memento — a handle the owner keeps, so that the scroll position and the row under the
//! keyboard survive a rebuild of the list. Nothing here hands out a lock.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use blinc_core::owner::Owner;
use blinc_core::reactive::{
    Computed, ReactiveGraph, Signal, computed, dispose_derived, dispose_signal, signal,
};
use blinc_layout::selector::ScrollRef;
use blinc_layout::widgets::scroll::{ScrollPhysics, SharedScrollPhysics};

use super::window::visible_rows;

/// No row: the value of the atomics below.
const NONE: usize = usize::MAX;

/// The scroll position of one list. Cheap to clone; clones share the position.
#[derive(Clone)]
pub struct ListPosition {
    pub(crate) scroll: ScrollRef,
    pub(crate) physics: SharedScrollPhysics,
    wanted: Arc<AtomicUsize>,
    cursor: Arc<AtomicUsize>,
    /// The key of the row under the keyboard, so a stale index can be told from a right one.
    cursor_key: Arc<Mutex<Option<String>>>,
    /// The key and row the keys went to, until the next build says whether the caller took it.
    going_to: Arc<Mutex<Option<(String, usize)>>>,
    window: Arc<Mutex<Option<WindowNode>>>,
}

/// What the window depends on besides the scroll offset: `(view, row height, count)`.
type Measure = (f32, f32, usize);

/// The derived list of rows in view. One per position, however often the list is built; it leaves
/// the reactive graph with the last clone of the position.
struct WindowNode {
    rows: Computed<Vec<usize>>,
    measure: Signal<Measure>,
    offset: Signal<f32>,
}

impl Drop for WindowNode {
    fn drop(&mut self) {
        dispose_derived(self.rows.derived_id());
        dispose_signal(self.measure.id());
        dispose_signal(self.offset.id());
    }
}

impl Default for ListPosition {
    fn default() -> Self {
        Self {
            scroll: ScrollRef::new(),
            physics: Arc::new(Mutex::new(ScrollPhysics::default())),
            wanted: Arc::new(AtomicUsize::new(NONE)),
            cursor: Arc::new(AtomicUsize::new(NONE)),
            cursor_key: Arc::new(Mutex::new(None)),
            going_to: Arc::new(Mutex::new(None)),
            window: Arc::new(Mutex::new(None)),
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
        *self
            .cursor_key
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = None;
    }

    /// Set the row under the keyboard, with its key.
    pub(super) fn set_cursor_on(&self, row: usize, key: &str) {
        self.cursor.store(row, Ordering::Relaxed);
        *self
            .cursor_key
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(key.to_string());
    }

    /// The key of the row under the keyboard, if it is known.
    pub(super) fn cursor_key(&self) -> Option<String> {
        self.cursor_key
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn lock_offset(&self) -> f32 {
        self.physics
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .offset_y
    }

    /// The keys went to `row`, named `key`; the next build shows whether the caller took it.
    pub(super) fn going_to(&self, key: &str, row: usize) {
        *self.going_to.lock().unwrap_or_else(PoisonError::into_inner) =
            Some((key.to_string(), row));
    }

    /// Where the keys went, once, if that is the row now selected.
    pub(super) fn arrived(&self, selected: Option<&str>) -> Option<usize> {
        let went = self
            .going_to
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()?;
        (Some(went.0.as_str()) == selected).then_some(went.1)
    }

    /// The rows in view, for these measures.
    pub(crate) fn window(&self, offset: f32, measure: Measure) -> Computed<Vec<usize>> {
        // Made once and kept across builds, so no build's scope may own it: `WindowNode` gives it back.
        let watch = Owner::detached(|| self.scroll.watch_y(offset));
        if watch.try_get().is_none_or(|y| (y - offset).abs() > 0.5) {
            watch.set(offset);
        }
        let mut node = self.window.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(node) = node.as_ref() {
            if node.measure.try_get() != Some(measure) {
                node.measure.set(measure);
            }
            return node.rows.clone();
        }
        let (shared, rows) = Owner::detached(|| {
            let shared = signal(measure);
            let rows = computed(move |graph: &ReactiveGraph| {
                let y = graph.get(watch).unwrap_or(offset);
                let (view, row_height, count) = graph.get(shared).unwrap_or(measure);
                visible_rows(y, view, row_height, count).collect::<Vec<usize>>()
            });
            (shared, rows)
        });
        *node = Some(WindowNode {
            rows: rows.clone(),
            measure: shared,
            offset: watch,
        });
        rows
    }

    /// Put the list at `offset` pixels from the top, also in the element already on screen.
    pub(crate) fn apply_offset(&self, offset: f32) {
        self.physics
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .offset_y = -offset;
        self.scroll.set_scroll_offset(0.0, offset);
    }
}
