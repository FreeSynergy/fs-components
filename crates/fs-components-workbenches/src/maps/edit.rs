//! Editing and creating.
//!
//! Pattern: State — `MapState::editing` is `Some` while the person edits: unsaved changes block
//! leaving, Cancel with changes only asks and discarding is another action in another place, and
//! leaving reads again, so what is shown is what is stored. Saving is in `save.rs`, deleting in
//! `delete.rs`.

use super::draft::{FieldKind, PlaceDraft};
use super::labels::MapLabels;
use super::shown::{Outcome, Shown};
use super::state::MapState;
use crate::error::{ErrorKind, SourceError};

/// An edit in progress.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceEdit {
    /// Which edit this is (counts up with every edit begun; a shell ties its text fields to it).
    pub id: u64,
    /// The place edited; `None` for a new one.
    pub key: Option<String>,
    /// The form as it is now.
    pub draft: PlaceDraft,
    /// Why the last save failed, if it did.
    pub error: Option<String>,
    /// The person pressed Cancel with unsaved changes, and has not answered yet.
    pub discard_asked: bool,
}

impl MapState {
    /// The edit in progress, if any.
    pub fn editing(&self) -> Option<&PlaceEdit> {
        self.editing.as_ref()
    }

    /// Is a place shown that can be changed (show an Edit button)?
    pub fn can_edit(&self) -> bool {
        self.capabilities.edit
            && self.editing.is_none()
            && matches!(&self.shown, Shown::Place { sheet, .. } if sheet.can_edit)
    }

    /// Can a new place be made now (show a New button)?
    pub fn can_create(&self) -> bool {
        self.capabilities.create && self.editing.is_none()
    }

    /// Start editing the place shown.
    pub fn begin_edit(&mut self) {
        if !self.can_edit() {
            return;
        }
        let Some(key) = self.selected().map(str::to_string) else {
            return;
        };
        self.delete_asked = None;
        let draft = self.source.draft(&key);
        self.start(Some(key), draft);
    }

    /// Start a new place at the centre of the map. With unsaved changes this is refused and says
    /// why.
    pub fn begin_create(&mut self) -> Outcome {
        if !self.capabilities.create {
            return Outcome::Done;
        }
        if self.blocked_by_edit() {
            return Outcome::Blocked;
        }
        self.delete_asked = None;
        let (lon, lat) = (self.position.lon(), self.position.lat());
        let draft = self.source.blank(lon, lat);
        self.start(None, draft);
        Outcome::Done
    }

    fn start(&mut self, key: Option<String>, draft: Result<PlaceDraft, SourceError>) {
        match draft {
            Ok(draft) => {
                self.notice = None;
                self.edit_counter += 1;
                self.editing = Some(PlaceEdit {
                    id: self.edit_counter,
                    key,
                    draft,
                    error: None,
                    discard_asked: false,
                });
            }
            Err(e) if e.kind == ErrorKind::NotFound => self.place_gone(),
            Err(e) => {
                self.notice = Some(MapLabels::fill(&self.labels.cannot_edit, &e.message));
            }
        }
    }

    /// The person typed in this field. A locked field stays as it is.
    pub fn set_field(&mut self, field: &str, text: &str) {
        let Some(edit) = &mut self.editing else {
            return;
        };
        let found = edit
            .draft
            .fields
            .iter_mut()
            .find(|f| f.key == field && !f.read_only);
        if let Some(found) = found
            && found.value != text
        {
            found.value = text.to_string();
            edit.discard_asked = false;
        }
    }

    /// The person pressed this yes-or-no field.
    pub fn toggle_field(&mut self, field: &str) {
        let flipped = self.editing.as_ref().and_then(|edit| {
            edit.draft
                .fields
                .iter()
                .find(|f| f.key == field && f.kind == FieldKind::Flag)
                .map(|f| if f.value == "true" { "false" } else { "true" })
        });
        if let Some(flipped) = flipped {
            self.set_field(field, flipped);
        }
    }

    /// Stop editing. With unsaved changes this only asks, however often it is pressed, so a double-click
    /// cannot discard: discarding is [`MapState::confirm_discard`], another action elsewhere.
    pub fn cancel_edit(&mut self) {
        let Some(edit) = &mut self.editing else {
            return;
        };
        if edit.draft.changed() {
            edit.discard_asked = true;
            self.notice = Some(self.labels.discard_confirm.clone());
            return;
        }
        self.leave_edit();
    }

    /// The second action: discards the changes. Does nothing unless asked.
    pub fn confirm_discard(&mut self) {
        if self.editing.as_ref().is_some_and(|e| e.discard_asked) {
            self.leave_edit();
        }
    }

    /// Answer the question of [`MapState::cancel_edit`] with no.
    pub fn keep_editing(&mut self) {
        if let Some(edit) = &mut self.editing
            && edit.discard_asked
        {
            edit.discard_asked = false;
            self.notice = None;
        }
    }

    fn leave_edit(&mut self) {
        let key = self.editing.take().and_then(|edit| edit.key);
        self.notice = None;
        // What is shown may be out of date by now, and the place may be gone.
        self.reload_places();
        if let Some(key) = key {
            self.show(&key);
        }
    }

    /// While there are unsaved changes, going elsewhere is refused with a notice.
    pub(super) fn blocked_by_edit(&mut self) -> bool {
        if self.editing.as_ref().is_some_and(|e| e.draft.changed()) {
            self.notice = Some(self.labels.unsaved.clone());
            return true;
        }
        self.editing = None;
        false
    }
}
