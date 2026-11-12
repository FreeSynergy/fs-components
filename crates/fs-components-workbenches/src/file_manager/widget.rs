//! The file manager as a thing a host can place in its own window.
//!
//! Pattern: Facade + Observer — the widget holds the shared state and builds the element. A search draws
//! only the work area again, so the search field keeps the keyboard; every other change asks the window
//! for a full rebuild, so nothing decided in a build goes stale. It has no window function: a workbench
//! does not run alone, and gives back a plain context (ADR 0056).
//!
//! No lock is held while elements are built: `on_state` runs its closure at once, so a guard alive in
//! the same statement would deadlock the interface thread. Views draw from a snapshot, and the state is
//! reached only through closures.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_core::{BlincContextState, Signal};
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::stateful_with_key;
use fs_components_primitives::{FieldSize, SearchField};
use fs_components_widgets::{Element, ListPosition};

use super::context::{FilesContext, Variant};
use super::labels::FileLabels;
use super::source::FileSource;
use super::state::FilesState;
use super::view::{
    PickHandler, band, command_row, frame, nav_row, notice_line, side, sizes, status_line,
    tabs_row, tree_pane, work,
};
use crate::input_slots::InputSlots;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// The file manager, ready to be built into a layout. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct FileManagerWidget {
    pub(super) id: u64,
    pub(super) state: Arc<Mutex<FilesState>>,
    field: Arc<Mutex<SearchField>>,
    typed_search: Arc<AtomicBool>,
    position: ListPosition,
    generation: Arc<AtomicU64>,
    slots: InputSlots,
    filter_slots: InputSlots,
    pub(super) on_pick: Arc<Mutex<Option<PickHandler>>>,
}

fn search_field(
    state: &Arc<Mutex<FilesState>>,
    typed: &Arc<AtomicBool>,
    id: u64,
    placeholder: &str,
) -> SearchField {
    let (state, typed) = (Arc::clone(state), Arc::clone(typed));
    SearchField::new(placeholder)
        .size(FieldSize::Body)
        .on_change(Arc::new(move |term| {
            typed.store(!term.is_empty(), Ordering::Relaxed);
            locked(&state).search(term);
            bump_list(id);
        }))
}

impl FileManagerWidget {
    /// The whole workbench over `source`.
    pub fn new(source: Arc<dyn FileSource>, labels: FileLabels) -> Self {
        Self::with_variant(source, labels, Variant::Full)
    }

    pub(super) fn with_variant(
        source: Arc<dyn FileSource>,
        labels: FileLabels,
        variant: Variant,
    ) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let placeholder = labels.search.clone();
        let state = Arc::new(Mutex::new(FilesState::new(source, labels, variant)));
        let typed_search = Arc::new(AtomicBool::new(false));
        let field = search_field(&state, &typed_search, id, &placeholder);
        Self {
            id,
            state,
            field: Arc::new(Mutex::new(field)),
            typed_search,
            position: ListPosition::new(),
            generation: Arc::new(AtomicU64::new(0)),
            slots: InputSlots::default(),
            filter_slots: InputSlots::default(),
            on_pick: Arc::new(Mutex::new(None)),
        }
    }

    /// Keep the arrangement of the columns (shown, hidden, order, widths) in `store` under `view`, the key of this view
    /// (ADR 0069), and take what the store holds from an earlier run.
    pub fn keep_columns_in(
        &self,
        store: std::sync::Arc<dyn fs_components_widgets::LayoutStore>,
        view: &str,
    ) {
        locked(&self.state).keep_columns_in(store, view);
    }

    /// Pin the user folders of the storage that are known by these names (see `FilesState::pin_by_names`).
    pub fn pin_by_names(&self, names: Vec<(fs_components_widgets::PinnedKind, String)>) {
        locked(&self.state).pin_by_names(names);
    }

    pub(super) fn set_on_pick(&self, handler: PickHandler) {
        *self.on_pick.lock().unwrap_or_else(PoisonError::into_inner) = Some(handler);
    }

    /// Run `f` with the state, for a host's menus and commands. A closure, so the lock cannot be held
    /// across calls. After a change made this way call [`FileManagerWidget::refresh`]. Do not call other
    /// methods of this widget from inside `f`.
    pub fn with_state<T>(&self, f: impl FnOnce(&mut FilesState) -> T) -> T {
        f(&mut locked(&self.state))
    }

    /// What is current, as a plain value for the host (ADR 0056). Provisional shape.
    pub fn context(&self) -> FilesContext {
        locked(&self.state).context()
    }

    /// The style sheet to add to the theme of the window.
    pub fn styles(&self) -> String {
        super::view::styles()
    }

    /// Draw again after the state was changed through [`FileManagerWidget::with_state`].
    pub fn refresh(&self) {
        rebuild();
    }

    /// The name element ids of this widget begin with: `<name>-<row key>`, `<name>-really`, ...
    pub fn element_name(&self) -> String {
        format!("fs-files-{}", self.id)
    }

    /// The search field; made anew when the term was cleared by going elsewhere, so it shows no text
    /// that no longer holds. `SearchField` has no way to clear it (listed in DESIGN.md).
    fn field_for(&self, snapshot: &FilesState) -> SearchField {
        let mut field = self.field.lock().unwrap_or_else(PoisonError::into_inner);
        if snapshot.term().is_empty() && self.typed_search.swap(false, Ordering::Relaxed) {
            *field = search_field(
                &self.state,
                &self.typed_search,
                self.id,
                &snapshot.labels().search,
            );
        }
        field.clone()
    }

    /// The element of the workbench, filling `width` x `height`. Call it inside the host's build
    /// function, not inside a live area.
    pub fn build(&self, width: f32, height: f32) -> impl Element + use<> {
        let snapshot = locked(&self.state).clone();
        let actions = self.actions();
        let name = self.element_name();
        let measured = sizes(&snapshot, width, height);
        let search = self.field_for(&snapshot).build();
        let tabs = (snapshot.variant() != super::context::Variant::Picker)
            .then(|| tabs_row(&snapshot, &name, width));
        let bar = nav_row(&snapshot, &name, &actions, search, width);
        let crumbs = command_row(&snapshot, &name, &actions, width);
        let notice = snapshot.notice().map(|n| notice_line(n, &name));
        let tree = tree_pane(&snapshot, &actions);
        let side_area = side(
            &snapshot,
            &name,
            &actions,
            if matches!(snapshot.side(), super::side::Side::Filter) {
                &self.filter_slots
            } else {
                &self.slots
            },
            (measured.side, measured.body, width),
        );
        let live = {
            let (state, position, generation, actions, name) = (
                Arc::clone(&self.state),
                self.position.clone(),
                Arc::clone(&self.generation),
                actions.clone(),
                name.clone(),
            );
            let (work_w, work_h) = (measured.work, measured.body);
            stateful_with_key::<NoState>(format!("{name}-work"))
                .deps([list_signal(self.id).id()])
                .on_state(move |_| {
                    let snapshot = locked(&state).clone();
                    let new_list = generation.swap(snapshot.list_generation(), Ordering::Relaxed)
                        != snapshot.list_generation();
                    work(
                        &snapshot,
                        &name,
                        &position,
                        &actions,
                        (work_w, work_h, new_list),
                    )
                })
        };
        let status =
            (snapshot.variant() != super::context::Variant::Picker).then(|| status_line(&snapshot));
        let questions = band(&snapshot, &name, &actions, &self.slots, width);
        frame(
            (measured, height),
            (tabs, bar, crumbs, notice),
            (tree, live, side_area),
            (status, questions),
        )
    }
}

pub(super) fn locked(state: &Mutex<FilesState>) -> MutexGuard<'_, FilesState> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

fn list_signal(id: u64) -> Signal<u64> {
    BlincContextState::get().use_signal_keyed(&format!("fs-files-{id}-list"), || 0u64)
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
