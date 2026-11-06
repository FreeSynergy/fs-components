//! The tasks workbench as a thing a host can place in its own window.
//!
//! Pattern: Facade + Observer — `TasksWidget` holds the shared state and builds the element. A search
//! draws the list area and the status view's lanes again, so the search field keeps the keyboard; every
//! other change asks the window for a full rebuild, so nothing decided in a build goes stale. It has no
//! window function: a workbench does not run alone, and gives back a plain context (ADR 0056).
//!
//! No lock is held while elements are built: `on_state` runs its closure at once, so a guard alive in
//! the same statement would deadlock the interface thread. Views draw from a snapshot, and the state
//! is reached only through closures.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_core::{BlincContextState, Signal};
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::stateful_with_key;
use fs_components_primitives::{FieldSize, SearchField};
use fs_components_widgets::sortable_list::SortableList;
use fs_components_widgets::{Element, ListPosition};

use super::board::Board;
use super::context::{TaskView, TasksContext};
use super::labels::TaskLabels;
use super::shown::Variant;
use super::source::TaskSource;
use super::state::TasksState;
use super::view::{Areas, InputSlots, board_area, detail, frame, list, toolbar, work_size};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// The tasks workbench, ready to be built into a layout. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct TasksWidget {
    pub(super) id: u64,
    pub(super) state: Arc<Mutex<TasksState>>,
    pub(super) board: Board,
    field: SearchField,
    position: ListPosition,
    generation: Arc<AtomicU64>,
    slots: InputSlots,
}

impl TasksWidget {
    /// The whole workbench over `source`.
    pub fn new(source: Arc<dyn TaskSource>, labels: TaskLabels) -> Self {
        Self::with_variant(source, labels, Variant::Full)
    }

    /// The compact "my next tasks" (ADR 0037): open tasks, list and detail, no changes, whatever the
    /// source can do.
    pub fn compact(source: Arc<dyn TaskSource>, labels: TaskLabels) -> Self {
        Self::with_variant(source, labels, Variant::Compact)
    }

    fn with_variant(source: Arc<dyn TaskSource>, labels: TaskLabels, variant: Variant) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let placeholder = labels.search.clone();
        let state = Arc::new(Mutex::new(TasksState::new(source, labels, variant)));
        let board = {
            let snapshot = locked(&state).clone();
            let target = Arc::clone(&state);
            Board::new(
                format!("fs-tasks-{id}-board"),
                &snapshot,
                Arc::new(move |key: &str| {
                    locked(&target).select(key);
                    rebuild();
                }),
            )
        };
        let field = {
            let (state, board) = (Arc::clone(&state), board.clone());
            SearchField::new(&placeholder)
                .size(FieldSize::Small)
                .on_change(Arc::new(move |term| {
                    let snapshot = {
                        let mut held = locked(&state);
                        held.search(term);
                        held.clone()
                    };
                    board.sync(&snapshot);
                    bump_list(id);
                }))
        };
        let widget = Self {
            id,
            state,
            board,
            field,
            position: ListPosition::new(),
            generation: Arc::new(AtomicU64::new(0)),
            slots: InputSlots::default(),
        };
        widget.listen_to_board();
        widget
    }

    /// Run `f` with the state, for a host's menus and commands. A closure, so the lock cannot be held
    /// across calls. After a change made this way call [`TasksWidget::refresh`]. Do not call other
    /// methods of this widget from inside `f`.
    pub fn with_state<T>(&self, f: impl FnOnce(&mut TasksState) -> T) -> T {
        let (answer, snapshot) = {
            let mut held = locked(&self.state);
            let answer = f(&mut held);
            (answer, held.clone())
        };
        self.board.sync(&snapshot);
        answer
    }

    /// What is current, as a plain value for the host (ADR 0056). Provisional shape.
    pub fn context(&self) -> TasksContext {
        locked(&self.state).context()
    }

    /// The status view's list, for a frontend's gestures (ADR 0037): `grab`, `hover`, `drop_grabbed`,
    /// `cancel` or `move_item`. What comes of it is [`TasksState::last_move`].
    pub fn board(&self) -> SortableList {
        self.board.list.clone()
    }

    /// The style sheet to add to the theme of the window.
    pub fn styles(&self) -> String {
        super::view::styles()
    }

    /// Draw again after the state was changed through [`TasksWidget::with_state`].
    pub fn refresh(&self) {
        rebuild();
    }

    /// The name element ids of this widget begin with: `<name>-<row key>`, `<name>-save`, ...
    pub fn element_name(&self) -> String {
        format!("fs-tasks-{}", self.id)
    }

    /// The element of the workbench, filling `width` x `height`. Call it inside the host's build
    /// function, not inside a live area.
    pub fn build(&self, width: f32, height: f32) -> impl Element + use<> {
        let snapshot = locked(&self.state).clone();
        let actions = self.actions();
        let name = self.element_name();
        let bar = toolbar(&snapshot, &name, &actions, self.field.build());
        let person = detail(&snapshot, &name, &actions, &self.slots);
        let (work_w, work_h) = work_size(width, height);
        let live = {
            let (state, position, generation, actions, name) = (
                Arc::clone(&self.state),
                self.position.clone(),
                Arc::clone(&self.generation),
                actions.clone(),
                name.clone(),
            );
            stateful_with_key::<NoState>(format!("{name}-work"))
                .deps([list_signal(self.id).id()])
                .on_state(move |_| {
                    let snapshot = locked(&state).clone();
                    // Another list took this one's place: its scroll position is void.
                    if generation.swap(snapshot.list_generation(), Ordering::Relaxed)
                        != snapshot.list_generation()
                    {
                        position.scroll_to_row(0);
                    }
                    list(&snapshot, &name, &position, &actions, work_w, work_h)
                })
        };
        let work = if snapshot.view() == TaskView::Status {
            board_area(
                &snapshot,
                &name,
                &actions,
                &self.board,
                live,
                (work_w, work_h),
            )
        } else {
            blinc_layout::div::div().child(live)
        };
        frame(
            Areas {
                toolbar: bar,
                work,
                detail: person,
            },
            width,
            height,
        )
    }
}

pub(super) fn locked(state: &Mutex<TasksState>) -> MutexGuard<'_, TasksState> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

fn list_signal(id: u64) -> Signal<u64> {
    BlincContextState::get().use_signal_keyed(&format!("fs-tasks-{id}-list"), || 0u64)
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
