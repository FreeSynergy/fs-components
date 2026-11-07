//! The compact variant: look, choose one file, give back a value (ADR 0037).
//!
//! Pattern: Facade over the same widget and state — the picker changes nothing, whatever the source can
//! do. Choosing a marked file is a second act at another place (ADR 0029). The host reads the value with
//! [`FilePicker::picked`] or is told through [`FilePicker::on_pick`].

use std::sync::Arc;

use super::ask::{Ask, Confirm};
use super::context::{FilesContext, PickedFile, Variant};
use super::labels::FileLabels;
use super::model::EntryKind;
use super::source::FileSource;
use super::state::FilesState;
use super::widget::FileManagerWidget;
use crate::file_manager::view::OnPick;

/// A file picker, ready to be built into a layout. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct FilePicker(FileManagerWidget);

impl FilePicker {
    /// The picker over `source`.
    pub fn new(source: Arc<dyn FileSource>, labels: FileLabels) -> Self {
        Self(FileManagerWidget::with_variant(
            source,
            labels,
            Variant::Picker,
        ))
    }

    /// Be told when a file was chosen.
    #[must_use]
    pub fn on_pick(self, handler: OnPick) -> Self {
        self.0.set_on_pick(handler);
        self
    }

    /// The file chosen, if one was.
    pub fn picked(&self) -> Option<PickedFile> {
        self.0.with_state(|s| s.picked().cloned())
    }

    /// What is current, as a plain value (ADR 0056).
    pub fn context(&self) -> FilesContext {
        self.0.context()
    }

    /// Run `f` with the state, as [`FileManagerWidget::with_state`] does.
    pub fn with_state<T>(&self, f: impl FnOnce(&mut FilesState) -> T) -> T {
        self.0.with_state(f)
    }

    /// The name element ids begin with.
    pub fn element_name(&self) -> String {
        self.0.element_name()
    }

    /// The style sheet to add to the theme of the window.
    pub fn styles(&self) -> String {
        self.0.styles()
    }

    /// The element, filling `width` x `height`. Call it inside the host's build function.
    pub fn build(&self, width: f32, height: f32) -> impl crate::file_manager::view::Elem + use<> {
        self.0.build(width, height)
    }
}

impl FilesState {
    /// The file chosen in the picker.
    pub fn picked(&self) -> Option<&PickedFile> {
        self.picked.as_ref()
    }

    /// Can what is selected be chosen (show the button)?
    pub fn can_choose(&self) -> bool {
        self.variant == Variant::Picker
            && !self.typing()
            && self
                .selected_entry()
                .is_some_and(|e| e.kind == EntryKind::File)
    }

    /// Choose what is selected. A marked file only asks.
    pub fn choose(&mut self) {
        if !self.can_choose() {
            return;
        }
        let Some(entry) = self.selected_entry().cloned() else {
            return;
        };
        if entry.is_marked() {
            self.ask = Ask::Confirm(Confirm::ChooseMarked {
                key: entry.key.clone(),
                name: self.named(&entry),
            });
        } else {
            self.picked_now(&entry.key, &entry.name, false);
        }
    }

    pub(super) fn picked_now(&mut self, key: &str, name: &str, marked: bool) {
        let name = self
            .selected_entry()
            .filter(|e| e.key == key)
            .map_or_else(|| name.to_string(), |e| e.name.clone());
        self.picked = Some(PickedFile {
            key: key.to_string(),
            name,
            marked,
        });
    }
}
