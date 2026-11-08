//! Editing, creating and deleting.
//!
//! Pattern: State — `PeopleState::editing` is `Some` while the person edits, the same rules as the
//! reader's editing: unsaved changes block leaving, Cancel with changes only asks and discarding is
//! another action in another place, a failed save keeps the draft and shows why, after a conflict the next Save writes knowingly. Deleting
//! is in `delete.rs`.

use super::labels::PeopleLabels;
use super::model::{DraftField, PersonDraft};
use super::shown::{Outcome, Shown};
use super::state::PeopleState;
use crate::error::ErrorKind;

/// An edit in progress.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonEdit {
    /// Which edit this is (counts up with every edit begun; a shell ties its text fields to it).
    pub id: u64,
    /// The person edited; `None` for a new one.
    pub key: Option<String>,
    /// The form as it is now.
    pub draft: PersonDraft,
    /// Why the last save failed, if it did.
    pub error: Option<String>,
    /// The person pressed Cancel with unsaved changes, and has not answered yet.
    pub discard_asked: bool,
}

impl PeopleState {
    /// The edit in progress, if any.
    pub fn editing(&self) -> Option<&PersonEdit> {
        self.editing.as_ref()
    }

    /// Is a person shown who can be changed (show an Edit button)?
    pub fn can_edit(&self) -> bool {
        self.capabilities.edit
            && self.editing.is_none()
            && matches!(self.shown, Shown::Person { .. })
    }

    /// Can a new person be made now (show a New button)?
    pub fn can_create(&self) -> bool {
        self.capabilities.create && self.editing.is_none()
    }

    /// Start editing the person shown.
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

    /// Start a new person. With unsaved changes this is refused and says why.
    pub fn begin_create(&mut self) -> Outcome {
        if !self.capabilities.create {
            return Outcome::Done;
        }
        if self.blocked_by_edit() {
            return Outcome::Blocked;
        }
        self.delete_asked = None;
        let draft = self.source.blank();
        self.start(None, draft);
        Outcome::Done
    }

    fn start(
        &mut self,
        key: Option<String>,
        draft: Result<PersonDraft, crate::error::SourceError>,
    ) {
        match draft {
            Ok(draft) => {
                self.notice = None;
                self.edit_counter += 1;
                self.editing = Some(PersonEdit {
                    id: self.edit_counter,
                    key,
                    draft,
                    error: None,
                    discard_asked: false,
                });
            }
            Err(e) => {
                self.notice = Some(PeopleLabels::fill(&self.labels.cannot_edit, &e.message));
            }
        }
    }

    /// What is wrong with the text of the field `key` now, in the source's words.
    pub fn problem(&self, key: &str) -> Option<String> {
        let found = self
            .editing
            .as_ref()?
            .draft
            .fields()
            .find(|f| f.key == key)?;
        if !found.changed() {
            return None; // what is stored is not judged, only what the person writes
        }
        self.source.check(found.check, &found.value)
    }

    /// The person typed in this field. Whether the problem shown under it changed: only then must
    /// the form be drawn again.
    pub fn typed(&mut self, field: &str, text: &str) -> bool {
        let before = self.problem(field);
        self.set_field(field, text);
        before != self.problem(field)
    }

    /// The person typed in this field.
    pub fn set_field(&mut self, field: &str, text: &str) {
        let Some(edit) = &mut self.editing else {
            return;
        };
        let found = edit
            .draft
            .groups
            .iter_mut()
            .flat_map(|g| g.fields.iter_mut())
            .find(|f| f.key == field);
        if let Some(found) = found
            && found.value != text
        {
            found.value = text.to_string();
            edit.discard_asked = false;
        }
    }

    /// Add an empty field to this group, if it may grow.
    pub fn add_field(&mut self, group: &str) {
        let Some(edit) = &mut self.editing else {
            return;
        };
        let id = edit.draft.fields().count();
        if let Some(group) = edit
            .draft
            .groups
            .iter_mut()
            .find(|g| g.key == group && g.can_add)
        {
            let mut field =
                DraftField::new(format!("{}/+{id}", group.key), group.label.clone(), "");
            field.added = true;
            field.check = group.check;
            group.fields.push(field);
        }
    }

    /// Save. Without changes nothing is written. On success the person is read again and the list
    /// follows; on failure, also a conflict, the draft stays and the reason is shown.
    pub fn save(&mut self) {
        let Some(edit) = &self.editing else {
            return;
        };
        let invalid = edit
            .draft
            .fields()
            .any(|f| f.changed() && self.source.check(f.check, &f.value).is_some());
        if invalid {
            let message = self.labels.fix_fields.clone();
            if let Some(edit) = &mut self.editing {
                edit.error = Some(message);
            }
            return;
        }
        if !edit.draft.changed() {
            let key = edit.key.clone();
            self.editing = None;
            self.notice = None;
            // Nothing of ours to write, but what is shown may be out of date by now.
            if let Some(key) = key {
                self.reload(self.loaded());
                self.show(&key);
            }
            if self.notice.is_none() {
                self.notice = Some(self.labels.no_changes.clone());
            }
            return;
        }
        let (key, draft) = (edit.key.clone(), edit.draft.clone());
        let done = match &key {
            Some(key) => self.source.save(key, &draft).map(|()| key.clone()),
            None => self.source.create(&draft),
        };
        match done {
            Ok(saved) => {
                let message = if key.is_some() {
                    &self.labels.saved
                } else {
                    &self.labels.created
                };
                let message = message.clone();
                self.reload(self.loaded());
                self.show(&saved);
                self.notice = Some(message);
            }
            Err(e) => {
                let conflict = e.kind == ErrorKind::Conflict;
                let reason = if conflict {
                    PeopleLabels::fill(&self.labels.conflict_confirm, &e.message)
                } else {
                    PeopleLabels::fill(&self.labels.not_saved, &e.message)
                };
                // After a conflict the next Save, with the revision found now, writes over the
                // other change: the person has been told and has the draft in view.
                let latest = key
                    .as_deref()
                    .filter(|_| conflict)
                    .and_then(|k| self.source.draft(k).ok())
                    .map(|d| d.revision);
                if let Some(edit) = &mut self.editing {
                    edit.error = Some(reason);
                    if let Some(revision) = latest {
                        edit.draft.revision = revision;
                    }
                }
            }
        }
    }

    /// Stop editing. With unsaved changes this only asks, however often it is pressed, so a double-click
    /// cannot discard: discarding is [`PeopleState::confirm_discard`], another action elsewhere.
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

    /// Answer the question of [`PeopleState::cancel_edit`] with no.
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
        // What is shown may be out of date by now, and the person may be gone.
        if let Some(key) = key {
            self.reload(self.loaded());
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
