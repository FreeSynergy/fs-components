//! The reader and its transitions.
//!
//! Pattern: State machine over a Strategy (`DataSource`): every transition is one method; the
//! view only reads.

use std::sync::Arc;

use fs_components_widgets::Document;

use crate::source::{DataSource, Entry, Hit, LinkTarget};

/// What the content area shows.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Shown {
    /// Nothing opened yet.
    #[default]
    Nothing,
    /// A document.
    Page {
        /// Its path.
        path: String,
        /// Its content.
        document: Document,
    },
    /// A document that could not be read.
    Failed {
        /// Its path.
        path: String,
        /// Why.
        reason: String,
    },
}

/// What following a link led to; the shell acts on [`Outcome::External`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// A document was opened.
    Opened,
    /// An outside address: open it with the system.
    External(String),
    /// Nothing there; a notice is shown.
    Missing,
}

/// The reader: a data source, what is shown, where we came from, and the search.
pub struct Reader {
    source: Arc<dyn DataSource>,
    entries: Vec<Entry>,
    shown: Shown,
    history: Vec<String>,
    term: String,
    hits: Vec<Hit>,
    notice: Option<String>,
}

impl Reader {
    /// A reader over `source`; lists everything once and opens the start page.
    pub fn new(source: Arc<dyn DataSource>) -> Self {
        let entries = source.entries("");
        let mut reader = Self {
            source,
            entries,
            shown: Shown::Nothing,
            history: Vec::new(),
            term: String::new(),
            hits: Vec::new(),
            notice: None,
        };
        let start = reader
            .source
            .start()
            .or_else(|| reader.entries.first().map(|e| e.path.clone()));
        if let Some(path) = start {
            reader.show(&path);
        }
        reader
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
            Shown::Page { path, .. } | Shown::Failed { path, .. } => Some(path),
            Shown::Nothing => None,
        }
    }

    /// Is there a page to go back to?
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

    /// A short message for the person (a link led nowhere), if any.
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    /// Open the document at `path`; the one shown now goes onto the way back.
    pub fn open(&mut self, path: &str) {
        if self.current_path() == Some(path) {
            return;
        }
        if let Some(current) = self.current_path() {
            self.history.push(current.to_string());
        }
        self.show(path);
    }

    /// Follow `href` as written in the document shown.
    pub fn follow(&mut self, href: &str) -> Outcome {
        let from = self.current_path().unwrap_or_default().to_string();
        match self.source.resolve(&from, href) {
            LinkTarget::Page(path) => {
                self.open(&path);
                Outcome::Opened
            }
            LinkTarget::External(url) => Outcome::External(url),
            LinkTarget::Missing(what) => {
                self.notice = Some(format!("There is nothing at \"{what}\"."));
                Outcome::Missing
            }
        }
    }

    /// Go back one page.
    pub fn back(&mut self) {
        if let Some(path) = self.history.pop() {
            self.show(&path);
        }
    }

    /// Search by name and content; an empty term ends the search.
    pub fn search(&mut self, term: &str) {
        self.term = term.trim().to_string();
        self.hits = if self.term.is_empty() {
            Vec::new()
        } else {
            self.source.find(&self.term)
        };
    }

    fn show(&mut self, path: &str) {
        self.notice = None;
        self.shown = match self.source.read(path) {
            Ok(document) => Shown::Page {
                path: path.to_string(),
                document,
            },
            Err(e) => Shown::Failed {
                path: path.to_string(),
                reason: e.to_string(),
            },
        };
    }
}
