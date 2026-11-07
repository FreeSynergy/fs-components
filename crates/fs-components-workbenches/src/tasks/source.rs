//! Where the tasks workbench gets its tasks.
//!
//! Pattern: Strategy — the workbench depends on this trait only (an Adapter over a
//! program's core implements it). The trait is synchronous and called on the interface thread, like
//! `DocumentSource`; this is provisional (see DESIGN.md).

use super::draft::TaskDraft;
use super::model::{RowQuery, TaskCapabilities, TaskColumn, TaskRow, TaskSheet};
use crate::error::SourceError;

/// A source of tasks.
pub trait TaskSource: Send + Sync {
    /// What it can do.
    fn capabilities(&self) -> TaskCapabilities;

    /// The columns of the status view, in order.
    fn columns(&self) -> Vec<TaskColumn>;

    /// Tasks matching `query`, in the source's order, skipping `offset`, at most `limit`.
    ///
    /// # Errors
    /// [`SourceError`] with a reason a person can read.
    fn rows(
        &self,
        query: &RowQuery,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<TaskRow>, SourceError>;

    /// One task, to be read.
    ///
    /// # Errors
    /// [`SourceError`]; [`ErrorKind::NotFound`](crate::ErrorKind::NotFound) if there is none.
    fn read(&self, key: &str) -> Result<TaskSheet, SourceError>;

    /// One task, as a form.
    ///
    /// # Errors
    /// [`SourceError`]; a source that cannot edit says `Unsupported`.
    fn draft(&self, _key: &str) -> Result<TaskDraft, SourceError> {
        Err(SourceError::unsupported("this source cannot edit"))
    }

    /// An empty form for a new task.
    ///
    /// # Errors
    /// [`SourceError`]; a source that cannot create says `Unsupported`.
    fn blank(&self) -> Result<TaskDraft, SourceError> {
        Err(SourceError::unsupported("this source cannot create"))
    }

    /// Store what changed in `draft`. Keep everything the form does not show. If the task has
    /// changed since `draft.revision`, answer [`ErrorKind::Conflict`](crate::ErrorKind::Conflict)
    /// and change nothing.
    ///
    /// # Errors
    /// [`SourceError`] with the reason; the workbench keeps the draft.
    fn save(&self, _key: &str, _draft: &TaskDraft) -> Result<(), SourceError> {
        Err(SourceError::unsupported("this source cannot edit"))
    }

    /// Make a new task from `draft`; returns the key.
    ///
    /// # Errors
    /// [`SourceError`] with the reason; the workbench keeps the draft.
    fn create(&self, _draft: &TaskDraft) -> Result<String, SourceError> {
        Err(SourceError::unsupported("this source cannot create"))
    }

    /// Delete a task.
    ///
    /// # Errors
    /// [`SourceError`] with the reason; the task is still there.
    fn delete(&self, _key: &str) -> Result<(), SourceError> {
        Err(SourceError::unsupported("this source cannot delete"))
    }

    /// Move a task to `column`: only what the column stands for changes. If the task has changed
    /// since `revision`, answer [`ErrorKind::Conflict`](crate::ErrorKind::Conflict) and change
    /// nothing.
    ///
    /// # Errors
    /// [`SourceError`] with the reason; the task stays where it is.
    fn move_to(
        &self,
        _key: &str,
        _column: &str,
        _revision: Option<&str>,
    ) -> Result<(), SourceError> {
        Err(SourceError::unsupported("this source cannot move"))
    }
}
