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
//! Workaround for the Blinc defect ec776dcf (signals of long-lived data are owned by the build scope); it goes away as
//! soon as Blinc itself keeps the field data detached from the build. Nothing else in the workspace (sources, tests,
//! examples, benches) may call the Blinc constructors of field data or `watch_y`: the test `tests/one_way.rs` of this
//! crate fails if one does (it cannot see through macros; a macro that expands to such a call is not caught).
//!
//! Threads: the signals of a keeper are given back by the thread that made them (the registry of scopes is per
//! thread, as Blinc's scopes are). A keeper dropped after that thread has ended leaves its entries in the pending list
//! for good (a few ids; nothing else is held). Call [`collect`] on the thread that builds the pages.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use blinc_core::owner::Owner;
use blinc_core::reactive::{EffectId, Signal, dispose_host_effect, global_graph};
use blinc_layout::selector::ScrollRef;
use blinc_layout::widgets::text_area::{SharedTextAreaState, text_area_state};
use blinc_layout::widgets::text_input::{
    SharedTextInputData, text_input_data, text_input_data_with_placeholder,
};

static NEXT: AtomicU64 = AtomicU64::new(1);
static PENDING: Mutex<Vec<u64>> = Mutex::new(Vec::new());
/// Effects that were to be given back while the graph was locked by the thread asking (inside an effect or a computed).
static PENDING_EFFECTS: Mutex<Vec<EffectId>> = Mutex::new(Vec::new());

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

    /// Make something under a scope of its own; answers it and its number (the number is what the registry of the
    /// feature `test-access` notes the field by).
    fn make_noted<T>(&self, f: impl FnOnce() -> T) -> (T, u64) {
        collect();
        let owner = Owner::default();
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        OWNERS.with(|o| o.borrow_mut().insert(id, owner.clone()));
        self.0
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(id);
        (owner.run(f), id)
    }

    /// The data of a one-line input.
    pub fn text_input(&self) -> SharedTextInputData {
        let (data, id) = self.make_noted(text_input_data);
        #[cfg(feature = "test-access")]
        crate::test_access::note_input(id, &data);
        #[cfg(not(feature = "test-access"))]
        let _ = id;
        data
    }

    /// The data of a one-line input showing `placeholder` while empty.
    pub fn text_input_with_placeholder(&self, placeholder: &str) -> SharedTextInputData {
        let (data, id) = self.make_noted(|| text_input_data_with_placeholder(placeholder));
        #[cfg(feature = "test-access")]
        crate::test_access::note_input(id, &data);
        #[cfg(not(feature = "test-access"))]
        let _ = id;
        data
    }

    /// The state of a many-line input.
    pub fn text_area(&self) -> SharedTextAreaState {
        let (state, id) = self.make_noted(text_area_state);
        #[cfg(feature = "test-access")]
        crate::test_access::note_area(id, &state);
        #[cfg(not(feature = "test-access"))]
        let _ = id;
        state
    }
}

/// Give back what dropped keepers held, for the keepers made on this thread. Called by every `FieldData` method
/// that makes something; call it by hand where none is made for a long time.
pub fn collect() {
    give_back_pending_effects();
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

/// Give an effect back to the reactive graph, safely from anywhere (a `Drop` too). `dispose_host_effect` takes the
/// graph lock directly, so called by a thread that is inside an effect or a computed (which hold that lock while they
/// run) it never returns. Here the lock is only tried: if it is free the effect goes now (and so does whatever was
/// queued before), if not the id is queued and goes at the next call of this function or of [`collect`], on any
/// thread. Never call `dispose_host_effect` directly from code that may run inside an effect.
pub fn dispose_effect_later(effect: EffectId) {
    PENDING_EFFECTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(effect);
    give_back_pending_effects();
}

fn give_back_pending_effects() {
    let free = match global_graph().try_lock() {
        Ok(_guard) => true,
        Err(std::sync::TryLockError::Poisoned(_)) => {
            // a poisoned graph cannot be used any more: forget what waits
            PENDING_EFFECTS
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clear();
            false
        }
        Err(std::sync::TryLockError::WouldBlock) => false,
    };
    if !free {
        return;
    }
    // the guard is gone here; between the try and the disposal another thread may hold the lock for a moment (the
    // disposal waits), but this thread holds none, so it cannot be waiting for itself
    let ids = std::mem::take(
        &mut *PENDING_EFFECTS
            .lock()
            .unwrap_or_else(PoisonError::into_inner),
    );
    for id in ids {
        dispose_host_effect(id);
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
