//! Moving a task to another column.
//!
//! Pattern: State — a move is one transition with one value as its answer, whichever way
//! it came (a button, the keyboard, a frontend's gesture). The source decides; the state only reads
//! again and says what happened. A move that failed leaves the task where the source has it. A press
//! on the column the task is in does nothing, so a double-click cannot move it twice. While the list
//! cannot be read nothing is known about a row's state, so a move is not tried then: it would answer
//! with a conflict nobody caused.

use super::labels::TaskLabels;
use super::shown::Shown;
use super::state::TasksState;
use crate::error::ErrorKind;

/// What came of a move.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MoveOutcome {
    /// The task moved.
    Moved {
        /// The key of the task.
        key: String,
        /// The column it left.
        from: String,
        /// The column it is in now.
        to: String,
    },
    /// It is in that column already; nothing was written.
    Unchanged,
    /// Refused: there are unsaved changes.
    Blocked,
    /// Refused, or it failed; the reason, in words. The task stays where it was.
    Refused(String),
    /// Someone changed the task in the meantime; nothing was written. The new state is shown.
    Conflict(String),
    /// The task no longer exists.
    Gone,
}

impl TasksState {
    /// Can tasks be moved at all?
    pub fn can_move(&self) -> bool {
        self.capabilities.move_tasks
    }

    /// What the last move came to.
    pub fn last_move(&self) -> Option<&MoveOutcome> {
        self.last_move.as_ref()
    }

    /// Move a task to `column`.
    pub fn move_task(&mut self, key: &str, column: &str) -> MoveOutcome {
        let outcome = self.moved(key, column);
        self.last_move = Some(outcome.clone());
        outcome
    }

    fn moved(&mut self, key: &str, column: &str) -> MoveOutcome {
        if !self.can_move() {
            return MoveOutcome::Refused(String::new());
        }
        if self.blocked_by_edit() {
            return MoveOutcome::Blocked;
        }
        if self.list_problem.is_some() {
            self.reload(self.loaded());
            if let Some(problem) = self.list_problem.clone() {
                self.notice = Some(TaskLabels::fill(&self.labels.not_moved, &problem));
                return MoveOutcome::Refused(problem);
            }
        }
        let (from, title, revision) = self.where_is(key);
        if from.as_deref() == Some(column) {
            return MoveOutcome::Unchanged;
        }
        match self.source.move_to(key, column, revision.as_deref()) {
            Ok(()) => {
                self.read_again(key);
                // It did move: the card stands there also if the list could not be read again.
                if self.list_problem.is_some() {
                    self.put_row(key, column);
                }
                let column_title = self.column_title(column);
                self.notice = Some(TaskLabels::fill_two(
                    &self.labels.moved,
                    &title,
                    &column_title,
                ));
                MoveOutcome::Moved {
                    key: key.to_string(),
                    from: from.unwrap_or_default(),
                    to: column.to_string(),
                }
            }
            Err(e) if e.kind == ErrorKind::NotFound => {
                self.reload(self.loaded());
                if self.selected() == Some(key) {
                    self.task_gone();
                } else {
                    self.notice = Some(self.labels.gone.clone());
                }
                MoveOutcome::Gone
            }
            Err(e) => {
                let conflict = e.kind == ErrorKind::Conflict;
                self.read_again(key);
                self.notice = Some(TaskLabels::fill(&self.labels.not_moved, &e.message));
                if conflict {
                    MoveOutcome::Conflict(e.message)
                } else {
                    MoveOutcome::Refused(e.message)
                }
            }
        }
    }

    /// Say why a card cannot be picked up.
    pub fn refuse_move(&mut self, key: &str) {
        let reason = self
            .rows()
            .iter()
            .find(|r| r.key == key)
            .and_then(|r| r.move_refusal.clone());
        if let Some(reason) = reason {
            self.notice = Some(TaskLabels::fill(&self.labels.not_moved, &reason));
        }
    }

    /// A card was put elsewhere in its own column: nothing to write, but it must be said.
    pub fn refuse_order(&mut self) {
        self.notice = Some(self.labels.order_is_fixed.clone());
    }

    /// Column, title and revision of a task, from what has been read.
    fn where_is(&self, key: &str) -> (Option<String>, String, Option<String>) {
        if let Some(row) = self.rows().iter().find(|r| r.key == key) {
            return (
                Some(row.column.clone()),
                row.title.clone(),
                row.revision.clone(),
            );
        }
        match &self.shown {
            Shown::Task { key: shown, sheet } if shown == key => (
                Some(sheet.column.clone()),
                sheet.title.clone(),
                sheet.revision.clone(),
            ),
            _ => (None, key.to_string(), None),
        }
    }

    fn read_again(&mut self, key: &str) {
        self.reload(self.loaded());
        if self.selected() == Some(key) {
            self.show(key);
        }
    }

    fn column_title(&self, column: &str) -> String {
        self.columns()
            .iter()
            .find(|c| c.key == column)
            .map_or_else(|| column.to_string(), |c| c.title.clone())
    }
}
