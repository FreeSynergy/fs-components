//! The state of the people workbench, testable without a window.
//!
//! Pattern: State machine over a Strategy (`PeopleSource`): every transition is one method; the
//! view only reads. Editing and deleting are in `edit.rs`. `Clone` is a cheap snapshot (the rows are
//! shared), so a view can draw from a copy and hold no lock while it builds elements.

use std::sync::Arc;

use super::edit::PersonEdit;
use super::labels::PeopleLabels;
use super::model::{PeopleCapabilities, PersonRow};
use super::shown::{Outcome, Shown, Variant};
use super::source::PeopleSource;
use crate::error::{ErrorKind, SourceError};

/// Rows read at a time.
pub const PAGE: usize = 200;

/// List, selection, search, and the edit in progress.
#[derive(Clone)]
pub struct PeopleState {
    pub(super) source: Arc<dyn PeopleSource>,
    pub(super) labels: Arc<PeopleLabels>,
    pub(super) capabilities: PeopleCapabilities,
    variant: Variant,
    rows: Arc<Vec<PersonRow>>,
    more: bool,
    term: String,
    list_generation: u64,
    pub(super) list_problem: Option<String>,
    pub(super) shown: Shown,
    pub(super) notice: Option<String>,
    pub(super) editing: Option<PersonEdit>,
    pub(super) edit_counter: u64,
    pub(super) delete_asked: Option<String>,
}

impl PeopleState {
    /// The workbench over `source`; lists the first page and shows the first person, unless that
    /// row is only to be opened when chosen.
    pub fn new(source: Arc<dyn PeopleSource>, labels: PeopleLabels, variant: Variant) -> Self {
        let capabilities = match variant {
            Variant::Compact => PeopleCapabilities::read_only(),
            _ => source.capabilities(),
        };
        let mut state = Self {
            source,
            labels: Arc::new(labels),
            capabilities,
            variant,
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
        };
        state.reload(PAGE);
        if let Some(key) = state.first_to_open(0) {
            state.show(&key);
        }
        state
    }

    /// The words.
    pub fn labels(&self) -> &PeopleLabels {
        &self.labels
    }

    /// Full or compact.
    pub fn variant(&self) -> Variant {
        self.variant
    }

    /// The rows loaded so far.
    pub fn rows(&self) -> &Arc<Vec<PersonRow>> {
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

    /// The key of the person selected, if any.
    pub fn selected(&self) -> Option<&str> {
        match &self.shown {
            Shown::Person { key, .. } | Shown::Failed { key, .. } => Some(key),
            Shown::Nothing => None,
        }
    }

    /// A short message for the person, if any.
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    /// Search; the list starts over. It does not leave the person shown, so it works during an edit.
    pub fn search(&mut self, term: &str) {
        let term = term.trim();
        if term == self.term {
            return;
        }
        self.term = term.to_string();
        self.list_generation += 1;
        self.reload(PAGE);
    }

    /// Load the next rows.
    pub fn load_more(&mut self) {
        if !self.more {
            return;
        }
        let offset = self.rows.len();
        match self.read_rows(offset, PAGE) {
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

    /// Show this person. With unsaved changes this is refused and says why.
    pub fn select(&mut self, key: &str) -> Outcome {
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
            Err(e) => {
                self.rows = Arc::new(Vec::new());
                self.more = false;
                self.list_failed(&e);
            }
        }
    }

    pub(super) fn loaded(&self) -> usize {
        self.rows.len()
    }

    /// `limit` rows from `offset`, and whether there are more: one row more is asked for.
    fn read_rows(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<PersonRow>, bool), SourceError> {
        let mut rows = self.source.rows(&self.term, offset, limit + 1)?;
        let more = rows.len() > limit;
        rows.truncate(limit);
        Ok((rows, more))
    }

    fn list_failed(&mut self, e: &SourceError) {
        self.list_problem = Some(PeopleLabels::fill(&self.labels.list_failed, &e.message));
    }

    /// Show `key` as it is now, ending any edit and any question.
    pub(super) fn show(&mut self, key: &str) {
        self.editing = None;
        self.delete_asked = None;
        self.notice = None;
        self.shown = match self.source.read(key) {
            Ok(sheet) => Shown::Person {
                key: key.to_string(),
                sheet,
            },
            Err(e) if e.kind == ErrorKind::NotFound => {
                self.person_gone();
                return;
            }
            Err(e) => Shown::Failed {
                key: key.to_string(),
                reason: PeopleLabels::fill(&self.labels.unreadable, &e.message),
            },
        };
    }

    /// The person shown no longer exists: they leave the list and the detail area says so.
    pub(super) fn person_gone(&mut self) {
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
