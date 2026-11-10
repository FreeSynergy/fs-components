//! The handle of the editor.
//!
//! Pattern: Facade + Observer — a clonable handle over the model, the text area and the live
//! redraw; every change is reported to the closure the caller handed in (Observer). The text
//! area sits outside the parts that are drawn again, so typing never loses focus. Gestures are
//! only button clicks; selecting and typing are the text area's own.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::widgets::text_area::{SharedTextAreaState, text_area_state};
use fs_components_widgets::Block;
use fs_components_widgets::live::Live;

use super::area;
use crate::labels::Labels;
use crate::model::{Counts, EditorModel, Selection};

/// Turns Markdown source into the blocks the preview draws (the caller's Markdown reader).
pub type Previewer = Arc<dyn Fn(&str) -> Vec<Block> + Send + Sync>;

/// What changed, as the listener hears it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    /// The text now.
    pub text: String,
    /// Differs from the saved text?
    pub dirty: bool,
    /// Characters and words now.
    pub counts: Counts,
}

/// Called after every change of the text.
pub type OnChange = Arc<dyn Fn(&Change) + Send + Sync>;

/// A Markdown editor with toolbar and preview. Cheap to clone; clones share everything.
#[derive(Clone)]
pub struct EditorView {
    pub(super) model: Arc<Mutex<EditorModel>>,
    pub(super) live: Live,
    pub(super) area: SharedTextAreaState,
    pub(super) labels: Labels,
    pub(super) id: String,
    pub(super) previewer: Previewer,
    pub(super) on_change: Option<OnChange>,
}

impl EditorView {
    /// An editor whose preview is made by `previewer`.
    pub fn new(previewer: Previewer) -> Self {
        Self {
            model: Arc::new(Mutex::new(EditorModel::default())),
            live: Live::new(),
            area: text_area_state(),
            labels: Labels::default(),
            id: "fs-te".into(),
            previewer,
            on_change: None,
        }
    }

    /// The words.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = labels;
        self
    }

    /// Element id prefix (`<id>-bold`, `-italic`, `-h1`, `-h2`, `-h3`, `-list`, `-link`, `-code`,
    /// `-quote`, `-undo`, `-redo`).
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Hear of every change of the text.
    #[must_use]
    pub fn on_change(mut self, listener: OnChange) -> Self {
        self.on_change = Some(listener);
        self
    }

    /// The text area, for a frontend that feeds it (input method, tests).
    pub fn area(&self) -> &SharedTextAreaState {
        &self.area
    }

    pub(super) fn lock(&self) -> MutexGuard<'_, EditorModel> {
        self.model.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// What the text area reports when the person types: take it as one step.
    pub fn typed(&self, value: &str) {
        let (_, selection) = area::read(&self.area);
        let changed = {
            let mut model = self.lock();
            let text = area::with_final_break(model.text(), value);
            model.type_text(&text, selection)
        };
        if changed {
            self.announce();
        }
    }

    /// Run `edit` on the model, with the area's selection; show the result in the area.
    pub(super) fn run(&self, edit: impl FnOnce(&mut EditorModel) -> bool) -> bool {
        let (value, selection) = area::read(&self.area);
        let (changed, text, now) = {
            let mut model = self.lock();
            let text = area::with_final_break(model.text(), &value);
            model.type_text(&text, selection);
            let changed = edit(&mut model);
            (changed, model.text().to_owned(), model.selection())
        };
        area::write(&self.area, &text, now);
        self.announce();
        changed
    }

    /// Tell the listener and draw toolbar, preview and counters again.
    pub(super) fn announce(&self) {
        let change = {
            let model = self.lock();
            Change {
                text: model.text().to_owned(),
                dirty: model.is_dirty(),
                counts: model.counts(),
            }
        };
        if let Some(listener) = &self.on_change {
            listener(&change);
        }
        self.live.refresh();
    }

    /// The selection as the model has it.
    pub(super) fn selection_now(&self) -> Selection {
        self.lock().selection()
    }
}
