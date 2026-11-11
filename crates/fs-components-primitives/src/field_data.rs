//! The one way to make the data of an input field and the watch of a scroll position.
//!
//! Pattern: Factory (a keeper that makes the data) with Resource-owner (it gives the signals back when the last
//! clone goes). Why: Blinc's `TextInputData`, `TextAreaState` and `ScrollRef::watch_y` make reactive signals when
//! they are made. Made while a page is being built they belong to that build's scope, the scope is disposed at the
//! next fresh build, and the data (kept by the program) lives on with dead signals: the field shows no typed text.
//! Here the data is made under a scope of its own, owned by no build, and that scope is disposed when the last
//! clone of the [`FieldData`] is dropped (never under the graph lock: the ids are queued and given back at the next
//! [`FieldData::text_input`], [`FieldData::text_area`] or [`collect`] on the thread that made them).
//!
//! Workaround for the Blinc defect ec776dcf (signals of long-lived data are owned by the build scope); it goes away
//! with fs-blinc-5 (`DataSignals`). Nothing else in the workspace may call `text_input_data()`,
//! `text_area_state()`, `TextInputData::new`, `TextAreaState::new`, `ScrollRef::new` or `watch_y`: the test
//! `tests/one_way.rs` of this crate fails if one does.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use blinc_core::owner::Owner;
use blinc_core::reactive::Signal;
use blinc_layout::selector::ScrollRef;
use blinc_layout::widgets::text_area::{SharedTextAreaState, text_area_state};
use blinc_layout::widgets::text_input::{
    SharedTextInputData, text_input_data, text_input_data_with_placeholder,
};

static NEXT: AtomicU64 = AtomicU64::new(1);
static PENDING: Mutex<Vec<u64>> = Mutex::new(Vec::new());

thread_local! {
    static OWNERS: RefCell<HashMap<u64, Owner>> = RefCell::new(HashMap::new());
}

#[derive(Default)]
struct Held(Mutex<Vec<u64>>);

impl Drop for Held {
    fn drop(&mut self) {
        let ids = std::mem::take(&mut *self.0.lock().unwrap_or_else(PoisonError::into_inner));
        if !ids.is_empty() {
            PENDING
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .extend(ids);
        }
    }
}

/// Keeps the signals of the field data made through it alive exactly as long as it (and its clones) live. Keep it
/// next to the data, in the struct that holds the data.
#[derive(Clone, Default)]
pub struct FieldData(Arc<Held>);

impl FieldData {
    /// A keeper that holds nothing yet.
    pub fn new() -> Self {
        Self::default()
    }

    fn make<T>(&self, f: impl FnOnce() -> T) -> T {
        collect();
        let owner = Owner::default();
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        OWNERS.with(|o| o.borrow_mut().insert(id, owner.clone()));
        self.0
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(id);
        owner.run(f)
    }

    /// The data of a one-line input.
    pub fn text_input(&self) -> SharedTextInputData {
        self.make(text_input_data)
    }

    /// The data of a one-line input showing `placeholder` while empty.
    pub fn text_input_with_placeholder(&self, placeholder: &str) -> SharedTextInputData {
        self.make(|| text_input_data_with_placeholder(placeholder))
    }

    /// The state of a many-line input.
    pub fn text_area(&self) -> SharedTextAreaState {
        self.make(text_area_state)
    }
}

/// Give back what dropped keepers held, for the keepers made on this thread. Called by every `FieldData` method
/// that makes something; call it by hand where none is made for a long time.
pub fn collect() {
    let ids: Vec<u64> =
        std::mem::take(&mut *PENDING.lock().unwrap_or_else(PoisonError::into_inner));
    if ids.is_empty() {
        return;
    }
    let mut foreign = Vec::new();
    for id in ids {
        // Taken out first: disposing runs outside the borrow.
        match OWNERS.with(|o| o.borrow_mut().remove(&id)) {
            Some(owner) => owner.dispose(),
            None => foreign.push(id),
        }
    }
    if !foreign.is_empty() {
        PENDING
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .extend(foreign);
    }
}

/// A scroll position that is not bound yet. It makes no signal; it is here so that [`scroll_ref`] and
/// [`scroll_watch`] are the one place that touches `ScrollRef::new` and `watch_y`.
pub fn scroll_ref() -> ScrollRef {
    ScrollRef::new()
}

/// The vertical offset of `scroll` as a signal that belongs to no build (made once, kept with the ref; a ref lives as
/// long as its widget, so this is one signal per widget).
pub fn scroll_watch(scroll: &ScrollRef, initial: f32) -> Signal<f32> {
    Owner::detached(|| scroll.watch_y(initial))
}

/// A text area made in the current build scope, for one that is shown in this build only and kept by nobody (a picture
/// of an editor). It goes with the build; never keep it across builds.
pub fn build_scoped_text_area() -> SharedTextAreaState {
    text_area_state()
}
