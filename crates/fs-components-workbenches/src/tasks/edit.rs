//! Editing and creating.
//!
//! Pattern: State — `TasksState::editing` is `Some` while the person edits, the same rules as the
//! other workbenches: unsaved changes block leaving, Cancel with changes only asks and discarding is
//! another action in another place, a failed save keeps the draft and shows why, after a conflict the next
//! Save writes knowingly, and a task deleted elsewhere keeps what was typed. A Save without changes, a
//! Cancel and a failed save read the task again.

use super::draft::TaskDraft;
use super::labels::TaskLabels;
use super::shown::{Outcome, Shown};
use super::state::TasksState;
use crate::error::{ErrorKind, SourceError};

/// An edit in progress.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskEdit {
    /// Which edit this is (counts up with every edit begun; a view ties its text fields to it).
    pub id: u64,
    /// The task edited; `None` for a new one.
    pub key: Option<String>,
    /// The form as it is now.
    pub draft: TaskDraft,
    /// Why the last save failed, if it did.
    pub error: Option<String>,
    /// The person pressed Cancel with unsaved changes, and has not answered yet.
    pub discard_asked: bool,
}

impl TasksState {
    /// The edit in progress, if any.
    pub fn editing(&self) -> Option<&TaskEdit> {
        self.editing.as_ref()
    }

    /// Is a task shown that can be changed (show an Edit button)?
    pub fn can_edit(&self) -> bool {
        self.capabilities.edit && self.editing.is_none() && matches!(self.shown, Shown::Task { .. })
    }

    /// Can a new task be made now (show a New button)?
    pub fn can_create(&self) -> bool {
        self.capabilities.create && self.editing.is_none()
    }

    /// Start editing the task shown.
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

    /// Start a new task. With unsaved changes this is refused and says why.
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

    fn start(&mut self, key: Option<String>, draft: Result<TaskDraft, SourceError>) {
        match draft {
            Ok(draft) => {
                self.notice = None;
                self.edit_counter += 1;
                self.editing = Some(TaskEdit {
                    id: self.edit_counter,
                    key,
                    draft,
                    error: None,
                    discard_asked: false,
                });
            }
            Err(e) if e.kind == ErrorKind::NotFound => self.task_gone(),
            Err(e) => {
                self.notice = Some(TaskLabels::fill(&self.labels.cannot_edit, &e.message));
            }
        }
    }

    /// What is wrong with the text of the field `key` now, in the source's words. Only what the person
    /// changed is judged, never what is stored.
    pub fn problem(&self, key: &str) -> Option<String> {
        let found = self
            .editing
            .as_ref()?
            .draft
            .fields
            .iter()
            .find(|f| f.key == key)?;
        if !found.changed() {
            return None;
        }
        self.source.check(&found.key, &found.value)
    }

    /// The person typed in this field. Whether the problem shown under it changed: only then must
    /// the form be drawn again.
    pub fn typed(&mut self, field: &str, text: &str) -> bool {
        let before = self.problem(field);
        self.set_field(field, text);
        before != self.problem(field)
    }

    /// The person filled in this field.
    pub fn set_field(&mut self, field: &str, text: &str) {
        let Some(edit) = &mut self.editing else {
            return;
        };
        let found = edit.draft.fields.iter_mut().find(|f| f.key == field);
        if let Some(found) = found
            && found.value != text
        {
            found.value = text.to_string();
            edit.discard_asked = false;
        }
    }

    /// Save. Without changes nothing is written. On success the task is read again and the list
    /// follows; on failure, also a conflict, the draft stays and the reason is shown.
    pub fn save(&mut self) {
        let Some(edit) = &self.editing else {
            return;
        };
        let invalid = edit
            .draft
            .fields
            .iter()
            .any(|f| f.changed() && self.source.check(&f.key, &f.value).is_some());
        if invalid {
            let message = self.labels.fix_fields.clone();
            if let Some(edit) = &mut self.editing {
                edit.error = Some(message);
            }
            return;
        }
        if !edit.draft.changed() {
            let key = edit.key.clone();
            self.leave_edit(key);
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
                    self.labels.saved.clone()
                } else {
                    self.labels.created.clone()
                };
                self.reload(self.loaded());
                self.show(&saved);
                self.notice = Some(message);
            }
            Err(e) if e.kind == ErrorKind::NotFound && key.is_some() => {
                // What the person typed is theirs: the task leaves the list, the draft stays.
                self.reload(self.loaded());
                let gone = self.labels.gone_draft.clone();
                if let Some(edit) = &mut self.editing {
                    edit.error = Some(gone);
                }
            }
            Err(e) => self.save_failed(key.as_deref(), &e),
        }
    }

    /// The draft stays and says why; what is around it is read again, and after a conflict the next
    /// Save, with the revision found now, writes over the other change: the person has been told.
    fn save_failed(&mut self, key: Option<&str>, e: &SourceError) {
        let conflict = e.kind == ErrorKind::Conflict;
        let reason = if conflict {
            TaskLabels::fill(&self.labels.conflict_confirm, &e.message)
        } else {
            TaskLabels::fill(&self.labels.not_saved, &e.message)
        };
        let latest = key
            .filter(|_| conflict)
            .and_then(|k| self.source.draft(k).ok())
            .map(|d| d.revision);
        self.reload(self.loaded());
        if let Some(edit) = &mut self.editing {
            edit.error = Some(reason);
            if let Some(revision) = latest {
                edit.draft.revision = revision;
            }
        }
    }

    /// Stop editing. With unsaved changes this only asks, however often it is pressed, so a double-click
    /// cannot discard: discarding is [`TasksState::confirm_discard`], another action elsewhere.
    pub fn cancel_edit(&mut self) {
        let Some(edit) = &mut self.editing else {
            return;
        };
        if edit.draft.changed() {
            edit.discard_asked = true;
            self.notice = Some(self.labels.discard_confirm.clone());
            return;
        }
        let key = edit.key.clone();
        self.leave_edit(key);
    }

    /// The second action: discards the changes. Does nothing unless asked.
    pub fn confirm_discard(&mut self) {
        if let Some(edit) = &self.editing
            && edit.discard_asked
        {
            let key = edit.key.clone();
            self.leave_edit(key);
        }
    }

    /// Answer the question of [`TasksState::cancel_edit`] with no.
    pub fn keep_editing(&mut self) {
        if let Some(edit) = &mut self.editing
            && edit.discard_asked
        {
            edit.discard_asked = false;
            self.notice = None;
        }
    }

    /// The edit is over; what is shown may be out of date by now, and the task may be gone.
    fn leave_edit(&mut self, key: Option<String>) {
        self.editing = None;
        self.notice = None;
        self.reload(self.loaded());
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
