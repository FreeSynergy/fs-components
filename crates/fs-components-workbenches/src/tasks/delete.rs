//! Deleting a task.
//!
//! Pattern: State — two different actions: asking and confirming. Pressing the first again only asks
//! again, so a double-click can never delete, whatever is drawn under the pointer by then. The
//! question is tied to that task; anything that shows another task drops it. A failure is kept and
//! shown, and the task stays; a task someone else deleted just leaves the list.

use super::labels::TaskLabels;
use super::shown::Shown;
use super::state::TasksState;
use crate::error::ErrorKind;

impl TasksState {
    /// Is a task shown that can be deleted (show a Delete button)?
    pub fn can_delete(&self) -> bool {
        self.capabilities.delete && self.editing.is_none() && self.selected().is_some()
    }

    /// Has the person pressed Delete, for the task selected?
    pub fn delete_asked(&self) -> bool {
        self.delete_asked.is_some() && self.delete_asked.as_deref() == self.selected()
    }

    /// The first press: asks whether the task selected should really go.
    pub fn ask_delete(&mut self) {
        if !self.can_delete() {
            return;
        }
        self.delete_asked = self.selected().map(str::to_string);
        self.notice = Some(TaskLabels::fill(&self.labels.delete_confirm, &self.title()));
    }

    /// The second action: deletes the task asked about. Does nothing unless asked.
    pub fn confirm_delete(&mut self) {
        if !self.can_delete() || !self.delete_asked() {
            return;
        }
        let Some(key) = self.selected().map(str::to_string) else {
            return;
        };
        self.delete_asked = None;
        let title = self.title();
        if let Err(e) = self.source.delete(&key) {
            if e.kind == ErrorKind::NotFound {
                self.task_gone();
            } else {
                self.notice = Some(TaskLabels::fill(&self.labels.not_deleted, &e.message));
            }
            return;
        }
        let place = self.rows().iter().position(|r| r.key == key).unwrap_or(0);
        self.reload(self.loaded());
        match self.first_to_open(place) {
            Some(next) => self.show(&next),
            None => self.shown = Shown::Nothing,
        }
        self.notice = Some(TaskLabels::fill(&self.labels.deleted, &title));
    }

    /// Answer the question of [`TasksState::ask_delete`] with no.
    pub fn keep(&mut self) {
        if self.delete_asked.take().is_some() {
            self.notice = None;
        }
    }

    pub(super) fn title(&self) -> String {
        match &self.shown {
            Shown::Task { sheet, .. } => sheet.title.clone(),
            Shown::Failed { key, .. } => key.clone(),
            Shown::Nothing => String::new(),
        }
    }
}
