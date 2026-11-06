//! The state of the tasks workbench, testable without a window.
//!
//! Pattern: State machine over a Strategy (`TaskSource`): every transition is one method; the view
//! only reads. Editing, deleting and moving have their own files. `Clone` is a cheap snapshot (the
//! rows are shared), so a view can draw from a copy and hold no lock while it builds elements.

use std::sync::Arc;

use super::context::{TaskView, TasksContext};
use super::edit::TaskEdit;
use super::labels::TaskLabels;
use super::model::{RowQuery, TaskCapabilities, TaskColumn, TaskRow};
use super::moving::MoveOutcome;
use super::shown::{Outcome, Shown, Variant};
use super::source::TaskSource;
use crate::error::{ErrorKind, SourceError};

/// Rows read at a time.
pub const PAGE: usize = 200;

/// List, selection, search, and the edit in progress.
#[derive(Clone)]
pub struct TasksState {
    pub(super) source: Arc<dyn TaskSource>,
    pub(super) labels: Arc<TaskLabels>,
    pub(super) capabilities: TaskCapabilities,
    variant: Variant,
    view: TaskView,
    columns: Arc<Vec<TaskColumn>>,
    rows: Arc<Vec<TaskRow>>,
    more: bool,
    term: String,
    list_generation: u64,
    pub(super) list_problem: Option<String>,
    pub(super) shown: Shown,
    pub(super) notice: Option<String>,
    pub(super) editing: Option<TaskEdit>,
    pub(super) edit_counter: u64,
    pub(super) delete_asked: Option<String>,
    pub(super) last_move: Option<MoveOutcome>,
    walked: Option<String>,
}

impl TasksState {
    /// The workbench over `source`; lists the first page and shows the first task, unless that row is
    /// only to be opened when chosen.
    pub fn new(source: Arc<dyn TaskSource>, labels: TaskLabels, variant: Variant) -> Self {
        let capabilities = match variant {
            Variant::Compact => TaskCapabilities::read_only(),
            _ => source.capabilities(),
        };
        let columns = Arc::new(source.columns());
        let mut state = Self {
            source,
            labels: Arc::new(labels),
            capabilities,
            variant,
            view: TaskView::List,
            columns,
            rows: Arc::new(Vec::new()),
            more: false,
            term: String::new(),
            list_generation: 0,
            list_problem: None,
            shown: Shown::Nothing,
            notice: None,
            editing: None,
            edit_counter: 0,
            delete_asked: None,
            last_move: None,
            walked: None,
        };
        state.reload(PAGE);
        if let Some(key) = state.first_to_open(0) {
            state.show(&key);
        }
        state
    }

    /// What is current, for the host (ADR 0056).
    pub fn context(&self) -> TasksContext {
        TasksContext {
            selected: self.selected().map(str::to_string),
            view: self.view,
        }
    }

    /// The words.
    pub fn labels(&self) -> &TaskLabels {
        &self.labels
    }

    /// Full or compact.
    pub fn variant(&self) -> Variant {
        self.variant
    }

    /// The view shown.
    pub fn view(&self) -> TaskView {
        self.view
    }

    /// Show the tasks this way. It leaves nothing: an edit stays open. The compact variant has only
    /// the list.
    pub fn set_view(&mut self, view: TaskView) {
        if self.variant == Variant::Full {
            self.view = view;
        }
    }

    /// The columns of the status view.
    pub fn columns(&self) -> &Arc<Vec<TaskColumn>> {
        &self.columns
    }

    /// The rows loaded so far.
    pub fn rows(&self) -> &Arc<Vec<TaskRow>> {
        &self.rows
    }

    /// Are there more rows than are loaded?
    pub fn has_more(&self) -> bool {
        self.more
    }

    /// The search text.
    pub fn term(&self) -> &str {
        &self.term
    }

    /// Counts up whenever another list takes the place of this one: its scroll position is void.
    pub fn list_generation(&self) -> u64 {
        self.list_generation
    }

    /// Why the list failed, if it did.
    pub fn list_problem(&self) -> Option<&str> {
        self.list_problem.as_deref()
    }

    /// What the detail area shows.
    pub fn shown(&self) -> &Shown {
        &self.shown
    }

    /// The key of the task selected, if any.
    pub fn selected(&self) -> Option<&str> {
        match &self.shown {
            Shown::Task { key, .. } | Shown::Failed { key, .. } => Some(key),
            Shown::Nothing => None,
        }
    }

    /// A short message for the person, if any.
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    /// Search; the list starts over. It does not leave the task shown, so it works during an edit.
    pub fn search(&mut self, term: &str) {
        let term = term.trim();
        if term == self.term {
            return;
        }
        self.term = term.to_string();
        self.list_generation += 1;
        self.walked = None;
        // The rows of another search must not stand here if this one cannot be read.
        self.rows = Arc::new(Vec::new());
        self.more = false;
        self.reload(PAGE);
    }

    /// Load the next rows. A failure keeps the rows, says why, and can be tried again.
    pub fn load_more(&mut self) {
        if !self.more {
            return;
        }
        match self.read_rows(self.rows.len(), PAGE) {
            Ok((rows, more)) => {
                let mut all = self.rows.as_ref().clone();
                all.extend(rows);
                self.rows = Arc::new(all);
                self.more = more;
                self.list_problem = None;
            }
            Err(e) => self.list_failed(&e),
        }
    }

    /// The keys of the list went to this row: it stands out, and nothing is read or opened, so walking
    /// over a marked row shows nothing of it (ADR 0029).
    pub fn walk(&mut self, key: &str) {
        self.walked = Some(key.to_string());
    }

    /// The row that stands out in the list: where the keys went, else the task selected.
    pub fn list_cursor(&self) -> Option<&str> {
        self.walked.as_deref().or_else(|| self.selected())
    }

    /// Show this task. With unsaved changes this is refused and says why.
    pub fn select(&mut self, key: &str) -> Outcome {
        self.walked = None;
        if self.selected() == Some(key) && self.editing.is_none() {
            return Outcome::Done;
        }
        if self.blocked_by_edit() {
            return Outcome::Blocked;
        }
        self.show(key);
        Outcome::Done
    }

    /// Read the first `count` rows (at least a page) again, as the list has changed.
    pub(super) fn reload(&mut self, count: usize) {
        match self.read_rows(0, count.max(PAGE)) {
            Ok((rows, more)) => {
                self.rows = Arc::new(rows);
                self.more = more;
                self.list_problem = None;
            }
            // What is drawn stays: a failed read says nothing about the tasks.
            Err(e) => self.list_failed(&e),
        }
    }

    /// A row is in this column now, whatever the list could say.
    pub(super) fn put_row(&mut self, key: &str, column: &str) {
        let mut rows = self.rows.as_ref().clone();
        if let Some(row) = rows.iter_mut().find(|r| r.key == key) {
            column.clone_into(&mut row.column);
        }
        self.rows = Arc::new(rows);
    }

    pub(super) fn loaded(&self) -> usize {
        self.rows.len()
    }

    /// `limit` rows from `offset`, and whether there are more: one row more is asked for.
    fn read_rows(&self, offset: usize, limit: usize) -> Result<(Vec<TaskRow>, bool), SourceError> {
        let mut query = RowQuery::default();
        query.term.clone_from(&self.term);
        query.open_only = self.variant == Variant::Compact;
        let mut rows = self.source.rows(&query, offset, limit + 1)?;
        let more = rows.len() > limit;
        rows.truncate(limit);
        Ok((rows, more))
    }

    fn list_failed(&mut self, e: &SourceError) {
        self.list_problem = Some(TaskLabels::fill(&self.labels.list_failed, &e.message));
    }

    /// Show `key` as it is now, ending any edit and any question.
    pub(super) fn show(&mut self, key: &str) {
        self.editing = None;
        self.delete_asked = None;
        self.notice = None;
        self.shown = match self.source.read(key) {
            Ok(sheet) => Shown::Task {
                key: key.to_string(),
                sheet,
            },
            Err(e) if e.kind == ErrorKind::NotFound => {
                self.task_gone();
                return;
            }
            Err(e) => Shown::Failed {
                key: key.to_string(),
                reason: TaskLabels::fill(&self.labels.unreadable, &e.message),
            },
        };
    }

    /// The task shown no longer exists: it leaves the list and the detail area says so.
    pub(super) fn task_gone(&mut self) {
        self.editing = None;
        self.delete_asked = None;
        self.reload(self.loaded());
        self.shown = Shown::Nothing;
        self.notice = Some(self.labels.gone.clone());
    }

    /// The row at `place`, or the last one, if the workbench may open it by itself.
    pub(super) fn first_to_open(&self, place: usize) -> Option<String> {
        self.rows
            .get(place)
            .or_else(|| self.rows.last())
            .filter(|row| !row.only_when_chosen)
            .map(|row| row.key.clone())
    }
}
