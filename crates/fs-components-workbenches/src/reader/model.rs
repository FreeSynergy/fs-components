//! The reader and its navigation.
//!
//! Pattern: State machine over a Strategy (`DocumentSource`): every transition is one method; the
//! view only reads. Editing is in `edit.rs`.

use std::sync::Arc;

use super::backup::Backup;
use super::edit::Editing;
use super::state::{Outcome, Shown};
use crate::labels::Labels;
use crate::source::{DocumentSource, Entry, Hit, LinkTarget};

/// The reader: a document source, what is shown, where we came from, and the search.
pub struct Reader {
    pub(super) source: Arc<dyn DocumentSource>,
    pub(super) labels: Labels,
    entries: Vec<Entry>,
    pub(super) shown: Shown,
    pub(super) history: Vec<String>,
    term: String,
    hits: Vec<Hit>,
    pub(super) list_problem: Option<String>,
    pub(super) search_problem: Option<String>,
    pub(super) editing: Option<Editing>,
    pub(super) edit_counter: u64,
    pub(super) backup: Backup,
    pub(super) notice: Option<String>,
    pub(super) export_target: Option<String>,
    pub(super) overwrite_ok: Option<(String, String)>,
}

impl Reader {
    /// A reader over `source`; lists everything once and opens the start document.
    pub fn new(source: Arc<dyn DocumentSource>, labels: Labels) -> Self {
        let mut reader = Self {
            source,
            labels,
            entries: Vec::new(),
            shown: Shown::Nothing,
            history: Vec::new(),
            term: String::new(),
            hits: Vec::new(),
            list_problem: None,
            search_problem: None,
            editing: None,
            edit_counter: 0,
            backup: Backup::new(),
            notice: None,
            export_target: None,
            overwrite_ok: None,
        };
        reader.refresh_entries();
        let start = reader
            .source
            .start()
            .or_else(|| reader.entries.first().map(|e| e.path.clone()));
        if let Some(path) = start {
            reader.show(&path);
        }
        reader
    }

    /// Read the list of documents again (after a change).
    pub fn refresh_entries(&mut self) {
        match self.source.entries("") {
            Ok(entries) => {
                self.entries = entries;
                self.list_problem = None;
            }
            Err(e) => {
                self.entries.clear();
                self.list_problem = Some(Labels::fill(&self.labels.list_failed, &e.message));
            }
        }
    }

    /// The words the reader shows.
    pub fn labels(&self) -> &Labels {
        &self.labels
    }

    /// Where Export writes to (the caller chooses; without one there is no Export button).
    pub fn set_export_target(&mut self, target: Option<String>) {
        self.export_target = target;
    }

    /// Every document, in path order.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// What the content area shows.
    pub fn shown(&self) -> &Shown {
        &self.shown
    }

    /// Path of the document shown, if any.
    pub fn current_path(&self) -> Option<&str> {
        match &self.shown {
            Shown::Document { path, .. } | Shown::Failed { path, .. } => Some(path),
            Shown::Nothing => None,
        }
    }

    /// Is there a document to go back to?
    pub fn can_go_back(&self) -> bool {
        !self.history.is_empty()
    }

    /// The search text.
    pub fn term(&self) -> &str {
        &self.term
    }

    /// Results of the search (empty while there is no search text).
    pub fn hits(&self) -> &[Hit] {
        &self.hits
    }

    /// Why the list or the last search failed, if it did.
    pub fn problem(&self) -> Option<&str> {
        self.list_problem
            .as_deref()
            .or(self.search_problem.as_deref())
    }

    /// A short message for the person, if any.
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    /// Open the document at `path`; the one shown now goes onto the way back.
    pub fn open(&mut self, path: &str) {
        if self.blocked_by_edit() || self.current_path() == Some(path) {
            return;
        }
        if let Some(current) = self.current_path() {
            self.history.push(current.to_string());
        }
        self.show(path);
    }

    /// Follow `href` as written in the document shown.
    pub fn follow(&mut self, href: &str) -> Outcome {
        if self.blocked_by_edit() {
            return Outcome::Blocked;
        }
        let from = self.current_path().unwrap_or_default().to_string();
        match self.source.resolve(&from, href) {
            LinkTarget::Document(path) => {
                self.open(&path);
                Outcome::Opened
            }
            LinkTarget::External(url) => Outcome::External(url),
            LinkTarget::Missing(what) => {
                self.notice = Some(Labels::fill(&self.labels.missing_target, &what));
                Outcome::Missing
            }
        }
    }

    /// Go back one step.
    pub fn back(&mut self) {
        if self.blocked_by_edit() {
            return;
        }
        if let Some(path) = self.history.pop() {
            self.show(&path);
        }
    }

    /// Search by name and content; an empty term ends the search.
    pub fn search(&mut self, term: &str) {
        self.term = term.trim().to_string();
        self.hits.clear();
        if self.term.is_empty() {
            self.search_problem = None;
            return;
        }
        match self.source.find(&self.term) {
            Ok(hits) => {
                self.hits = hits;
                self.search_problem = None;
            }
            Err(e) => {
                self.search_problem = Some(Labels::fill(&self.labels.search_failed, &e.message));
            }
        }
    }

    pub(super) fn show(&mut self, path: &str) {
        self.editing = None;
        self.overwrite_ok = None;
        self.notice = None;
        self.shown = match self.source.read(path) {
            Ok(document) => Shown::Document {
                path: path.to_string(),
                document,
            },
            Err(e) => Shown::Failed {
                path: path.to_string(),
                reason: Labels::fill(&self.labels.unreadable, &e.message),
            },
        };
    }
}
