//! The people workbench as a thing a program can place in its own window.
//!
//! Pattern: Facade + Observer — `PeopleWidget` holds the shared state and builds the element. A
//! search draws the list area again through a signal, so the search field keeps the keyboard;
//! every other change asks the window for a full rebuild, because `cn` elements built inside a live
//! area get new keys and their old callbacks keep running (see the Blinc notes in `DESIGN.md`).
//!
//! No lock is held while elements are built: `on_state` runs its closure at once, so a guard alive in
//! the same statement would deadlock the interface thread. Views draw from a snapshot.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_cn::components::button::ButtonVariant;
use blinc_core::{BlincContextState, Signal};
use blinc_layout::div::div;
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::stateful_with_key;
use fs_components_primitives::SearchField;
use fs_components_widgets::{Element, ListPosition};

use super::labels::PeopleLabels;
use super::shown::Variant;
use super::source::PeopleSource;
use super::state::PeopleState;
use super::view::{Areas, InputSlots, button_for, detail, frame, list, list_size};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// The people workbench, ready to be built into a layout. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct PeopleWidget {
    id: u64,
    pub(super) state: Arc<Mutex<PeopleState>>,
    field: SearchField,
    position: ListPosition,
    generation: Arc<AtomicU64>,
    slots: InputSlots,
}

impl PeopleWidget {
    /// The whole workbench over `source`.
    pub fn new(source: Arc<dyn PeopleSource>, labels: PeopleLabels) -> Self {
        Self::with_variant(source, labels, Variant::Full)
    }

    /// The compact variant (ADR 0037): list and detail, no editing, whatever the source can do.
    pub fn compact(source: Arc<dyn PeopleSource>, labels: PeopleLabels) -> Self {
        Self::with_variant(source, labels, Variant::Compact)
    }

    fn with_variant(source: Arc<dyn PeopleSource>, labels: PeopleLabels, variant: Variant) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let placeholder = labels.search.clone();
        let state = Arc::new(Mutex::new(PeopleState::new(source, labels, variant)));
        let field = {
            let state = Arc::clone(&state);
            SearchField::new(&placeholder).on_change(Arc::new(move |term| {
                locked(&state).search(term);
                bump_list(id);
            }))
        };
        Self {
            id,
            state,
            field,
            position: ListPosition::new(),
            generation: Arc::new(AtomicU64::new(0)),
            slots: InputSlots::default(),
        }
    }

    /// Run `f` with the state, for a program's menus and commands. A closure, so the lock cannot be
    /// held across calls. After a change made this way call [`PeopleWidget::refresh`]. Do not call
    /// other methods of this widget from inside `f`.
    pub fn with_state<T>(&self, f: impl FnOnce(&mut PeopleState) -> T) -> T {
        f(&mut locked(&self.state))
    }

    /// The style sheet to add to the theme of the window.
    pub fn styles(&self) -> String {
        super::view::styles()
    }

    /// Draw again after the state was changed through [`PeopleWidget::with_state`].
    pub fn refresh(&self) {
        rebuild();
    }

    /// The element of the workbench, filling `width` x `height`. Call it inside the program's build
    /// function, not inside a live area.
    pub fn build(&self, width: f32, height: f32) -> impl Element + use<> {
        let snapshot = locked(&self.state).clone();
        let actions = self.actions();
        let name = format!("fs-people-{}", self.id);
        let with_more = snapshot.has_more();
        let search = {
            let mut bar = div()
                .class("fs-people-search-row")
                .flex_row()
                .items_center()
                .flex_1()
                .child(
                    div()
                        .id(format!("{name}-search"))
                        .class("fs-people-search-field")
                        .flex_1()
                        .child(self.field.build()),
                );
            if snapshot.can_create() {
                bar = bar.child(button_for(
                    &name,
                    "create",
                    &snapshot.labels().create,
                    ButtonVariant::Primary,
                    &actions.create,
                ));
            }
            bar
        };
        let more = with_more.then(|| {
            div().class("fs-people-more").flex_row().child(button_for(
                &name,
                "more",
                &snapshot.labels().more,
                ButtonVariant::Secondary,
                &actions.more,
            ))
        });
        let (list_w, list_h) = list_size(width, height, with_more);
        let live = {
            let (state, position, generation, actions, name) = (
                Arc::clone(&self.state),
                self.position.clone(),
                Arc::clone(&self.generation),
                actions.clone(),
                name.clone(),
            );
            stateful_with_key::<NoState>(format!("{name}-list-area"))
                .deps([list_signal(self.id).id()])
                .on_state(move |_| {
                    let snapshot = locked(&state).clone();
                    // Another list took this one's place: its scroll position is void.
                    if generation.swap(snapshot.list_generation(), Ordering::Relaxed)
                        != snapshot.list_generation()
                    {
                        position.scroll_to_row(0);
                    }
                    list(&snapshot, &name, &position, &actions, list_w, list_h)
                })
        };
        let person = detail(&snapshot, &name, &actions, &self.slots);
        frame(
            Areas {
                search,
                list: live,
                more,
                detail: person,
            },
            width,
            height,
        )
    }

    /// The name element ids of this widget begin with: `<name>-<row key>`, `<name>-save`, ...
    pub fn element_name(&self) -> String {
        format!("fs-people-{}", self.id)
    }
}

pub(super) fn locked(state: &Mutex<PeopleState>) -> MutexGuard<'_, PeopleState> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

fn list_signal(id: u64) -> Signal<u64> {
    BlincContextState::get().use_signal_keyed(&format!("fs-people-{id}-list"), || 0u64)
}

fn bump_list(id: u64) {
    if BlincContextState::is_initialized() {
        list_signal(id).update(|v| v + 1);
    }
}

/// Ask the window to build everything again. Before the runtime is up there is nothing to draw.
pub(super) fn rebuild() {
    if BlincContextState::is_initialized() {
        BlincContextState::get().request_rebuild();
    }
}
