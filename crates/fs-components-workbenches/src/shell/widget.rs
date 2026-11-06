//! The reader as a thing a program can place in its own window.
//!
//! Pattern: Facade + Observer — `ReaderWidget` holds the shared state and builds the element; two
//! counters (signals) tell the two live areas, sidebar body and content, to draw again after a
//! change. A program owns the window, menus and gestures (ADR 0005, 0037) and calls
//! [`ReaderWidget::build`] from its own build function; [`launch`](super::launch) is only the
//! convenience of a window with nothing else in it.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_core::BlincContextState;
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::stateful;
use blinc_layout::widgets::text_area::SharedTextAreaState;
use fs_components_primitives::SearchField;
use fs_components_widgets::Element;
use fs_components_widgets::open_external;

use crate::labels::Labels;
use crate::reader::{DraftStore, Outcome, Reader};
use crate::source::DocumentSource;
use crate::view::{Actions, content, editor_rows, frame, new_area, side_body, tree_nodes};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// The reader, ready to be built into a layout. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct ReaderWidget {
    id: u64,
    reader: Arc<Mutex<Reader>>,
    field: SearchField,
    area: Arc<Mutex<Option<AreaSlot>>>,
    keys: bool,
}

/// The text area of the edit in progress: tied to that edit (its id), so an edit of another
/// document never shows this one's text.
struct AreaSlot {
    edit: u64,
    state: SharedTextAreaState,
}

impl ReaderWidget {
    /// A reader over `source` with `labels`; `export_target` switches Export on.
    pub fn new(
        source: Arc<dyn DocumentSource>,
        labels: Labels,
        export_target: Option<String>,
    ) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let mut reader = Reader::new(source, labels);
        reader.set_export_target(export_target);
        let placeholder = reader.labels().search.clone();
        let reader = Arc::new(Mutex::new(reader));
        let field = {
            let reader = Arc::clone(&reader);
            SearchField::new(&placeholder).on_change(Arc::new(move |term| {
                locked(&reader).search(term);
                bump(id, "side");
            }))
        };
        Self {
            id,
            reader,
            field,
            area: Arc::new(Mutex::new(None)),
            keys: true,
        }
    }

    /// Back the drafts of edits up into `store` (see [`DraftStore`]): after a pause in typing
    /// (half a second by default), at least every five seconds while typing goes on; a failed
    /// backup is retried and shown. A small thread does the timing, writes outside the reader's
    /// lock, redraws when the message about a failure changes, and ends with the widget. The
    /// window's global state keeps a built widget alive, so closing the window does **not** drop
    /// the reader and flush: the loss on a close is at most the last pause (or interval).
    #[must_use]
    pub fn drafts(self, store: Arc<dyn DraftStore>) -> Self {
        self.with_reader(|r| r.set_draft_store(Some(store)));
        let (weak, id) = (Arc::downgrade(&self.reader), self.id);
        std::thread::spawn(move || {
            while let Some(reader) = weak.upgrade() {
                let (job, before) = {
                    let mut r = locked(&reader);
                    (r.backup_job(), r.backup_problem().map(str::to_string))
                };
                if let Some(job) = job {
                    let result = job.run(); // may be slow: no lock held
                    let mut r = locked(&reader);
                    r.backup_done(&job, result);
                    if r.backup_problem().map(str::to_string) != before {
                        drop(r);
                        bump(id, "page");
                    }
                }
                drop(reader);
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
        });
        self
    }

    /// Back up after `pause` without typing and at least every `at_most` while typing goes on
    /// (call before [`ReaderWidget::drafts`]).
    #[must_use]
    pub fn backup_timing(self, pause: std::time::Duration, at_most: std::time::Duration) -> Self {
        self.with_reader(|r| r.set_backup_timing(pause, at_most));
        self
    }

    /// Run `f` with the state, for a program's menus and commands (open, back, save ...). A
    /// closure, so the lock cannot be held across calls (two guards in one statement would
    /// deadlock the interface thread). Do not call `with_reader` (or anything else of the widget
    /// that takes the lock) from inside `f`: that deadlocks. After a change made this way call
    /// [`ReaderWidget::refresh`].
    pub fn with_reader<T>(&self, f: impl FnOnce(&mut Reader) -> T) -> T {
        f(&mut locked(&self.reader))
    }

    /// Switch the reader's own keys (Alt+Left back, Ctrl+S save) off or on (default on). Keys
    /// reach every element of a window, so a program with several readers, or its own shortcuts
    /// (ADR 0037: gestures and keys belong to the frontend), switches them off and calls the
    /// reader itself.
    #[must_use]
    pub fn keys(mut self, on: bool) -> Self {
        self.keys = on;
        self
    }

    /// Draw the areas again after the state was changed through [`ReaderWidget::with_reader`].
    pub fn refresh(&self) {
        bump(self.id, "page");
        bump(self.id, "side");
    }

    /// The element of the reader, filling `width` x `height`. Call it inside the program's build
    /// function (it keeps its state in the framework's keyed state).
    ///
    /// **For hosts:** make the `ReaderWidget` once and keep it (clone it into the build closure);
    /// calling `build` again on the same widget is fine (its keys belong to the widget). Do not
    /// create a new `ReaderWidget` inside a build function or inside a live area: every new one
    /// gets new keys, the old one's callbacks, signals and backup thread keep running, and the
    /// state of the old one is lost. Inside the reader the sidebar rebuilds its tree with each
    /// redraw, which gives the tree new framework keys (see DESIGN.md).
    pub fn build(&self, width: f32, height: f32) -> impl Element + use<> {
        let (id, reader) = (self.id, &self.reader);
        let page_v = state(id, "page");
        let side_v = state(id, "side");
        let actions = self.actions();
        let side = {
            let (reader, actions) = (Arc::clone(reader), actions.clone());
            stateful::<NoState>()
                .deps([side_v.id()])
                .on_state(move |_| {
                    let reader = locked(&reader);
                    let tree = tree_nodes(reader.entries());
                    side_body(&reader, &tree, &actions)
                })
        };
        let main = {
            let (reader, actions, this) = (Arc::clone(reader), actions.clone(), self.clone());
            stateful::<NoState>()
                .deps([page_v.id()])
                .on_state(move |_| {
                    let reader = locked(&reader);
                    let area = this.sync_area(&reader);
                    let rows = editor_rows(height);
                    content(&reader, &actions, None, area.as_ref(), rows)
                })
        };
        let (back, save) = (Arc::clone(&actions.back), Arc::clone(&actions.save));
        // Esc is not handled: it reaches this element from the text area and the search field too,
        // and must never throw away a draft.
        let ui = frame(self.field.build(), side, main, width, height);
        if self.keys {
            ui.on_key_down(move |e| match e.key_code {
                0x25 if e.alt => back(),
                0x53 if e.ctrl => save(),
                _ => {}
            })
        } else {
            ui
        }
    }

    /// The text area for the edit in progress (none when not editing). A new one is made when the
    /// edit is another one, or when the draft was changed from outside (through
    /// [`ReaderWidget::with_reader`]) and the area no longer holds it.
    fn sync_area(&self, reader: &Reader) -> Option<SharedTextAreaState> {
        let mut slot = self.area.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(editing) = reader.editing() else {
            *slot = None;
            return None;
        };
        let current = slot.as_ref().filter(|a| a.edit == editing.id).filter(|a| {
            let shown = a
                .state
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .value();
            editing.restore_form(&shown) == editing.draft
        });
        if current.is_none() {
            *slot = Some(AreaSlot {
                edit: editing.id,
                state: new_area(&editing.draft),
            });
        }
        slot.as_ref().map(|a| a.state.clone())
    }

    /// What the text area of the edit in progress holds now (none when not editing); makes the
    /// area match the state first, as a redraw does.
    pub fn editor_text(&self) -> Option<String> {
        let area = self.sync_area(&locked(&self.reader))?;
        let text = area.lock().unwrap_or_else(PoisonError::into_inner).value();
        Some(text)
    }

    /// The callbacks that change the shared reader and ask the areas to draw again.
    fn actions(&self) -> Actions {
        let id = self.id;
        let simple = |change: fn(&mut Reader)| -> Arc<dyn Fn() + Send + Sync> {
            let reader = Arc::clone(&self.reader);
            Arc::new(move || {
                change(&mut locked(&reader));
                bump(id, "page");
                bump(id, "side");
            })
        };
        let (open_r, follow_r, draft_r) = (
            Arc::clone(&self.reader),
            Arc::clone(&self.reader),
            Arc::clone(&self.reader),
        );
        Actions {
            open: Arc::new(move |path| {
                locked(&open_r).open(path);
                bump(id, "page");
            }),
            follow: Arc::new(move |href| {
                let outcome = locked(&follow_r).follow(href);
                if let Outcome::External(url) = outcome {
                    open_external(&url);
                }
                bump(id, "page");
                bump(id, "side"); // the tree marks the page that is now open
            }),
            back: simple(Reader::back), // draws both areas (the tree marks the page)
            edit: simple(Reader::begin_edit),
            // Typing is remembered but does not redraw: the text area keeps its own cursor.
            draft: Arc::new(move |text| locked(&draft_r).set_draft(text)),
            save: simple(Reader::save),
            cancel: simple(Reader::cancel_edit),
            export: simple(Reader::export),
            restore: simple(Reader::restore_backup),
            discard: simple(Reader::discard_backup),
        }
    }
}

fn locked(reader: &Mutex<Reader>) -> MutexGuard<'_, Reader> {
    reader.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A signal, not a `State`: `State::update` does not reach the areas of a window when a picture
/// was drawn earlier in the process (the picture sets the reactive state up without the notifier),
/// `Signal::update` does in both orders.
fn state(id: u64, area: &str) -> blinc_core::Signal<u64> {
    BlincContextState::get().use_signal_keyed(&format!("fs-rd-{id}-{area}"), || 0u64)
}

/// Count up the signal of `area`, which makes the areas depending on it draw again.
fn bump(id: u64, area: &str) {
    state(id, area).update(|v| v + 1);
}
