//! The state of the file manager, testable without a window.
//!
//! Pattern: State machine over a Strategy (`FileSource`): every transition is one method; the view only
//! reads. Navigating, selecting, asking and transfers have their own files. `Clone` is a cheap snapshot
//! (the rows are shared), so a view can draw from a copy and hold no lock while it builds elements.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::ask::Ask;
use super::command::Carrying;
use super::context::{FileView, FilesContext, PickedFile, Variant};
use super::error::{FileError, FileProblem};
use super::filter::FilterDraft;
use super::labels::FileLabels;
use super::model::{FileEntry, FolderInfo, ListFilter, PageMark, SortOrder};
use super::reports::{FileCapabilities, RescanSummary};
use super::select::Selected;
use super::side::{Place, Side, TransferRow};
use super::source::FileSource;

/// Rows read at a time (the source's pages are put together to this many).
pub const PAGE: usize = 200;

/// A short message for the person.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notice {
    /// It went well, or there is something to know.
    Info(String),
    /// It did not work; the kind says how.
    Problem(FileProblem, String),
}

impl Notice {
    /// The words.
    pub fn text(&self) -> &str {
        match self {
            Self::Info(text) | Self::Problem(_, text) => text,
        }
    }
}

/// Folders and files, the selection, and what is being asked.
#[derive(Clone)]
pub struct FilesState {
    pub(super) source: Arc<dyn FileSource>,
    pub(super) labels: Arc<FileLabels>,
    pub(super) capabilities: FileCapabilities,
    pub(super) variant: Variant,
    pub(super) view: FileView,
    pub(super) place: Place,
    pub(super) path: Vec<FolderInfo>,
    pub(super) children: HashMap<Option<String>, (Vec<FolderInfo>, bool)>,
    pub(super) opened: HashSet<String>,
    pub(super) tree_problem: Option<String>,
    pub(super) rows: Arc<Vec<FileEntry>>,
    pub(super) unreadable: Arc<Vec<String>>,
    pub(super) next: Option<PageMark>,
    pub(super) term: String,
    pub(super) filter: ListFilter,
    pub(super) filter_draft: FilterDraft,
    pub(super) sort: SortOrder,
    pub(super) list_generation: u64,
    pub(super) list_problem: Option<String>,
    pub(super) selected: Option<Selected>,
    pub(super) notice: Option<Notice>,
    pub(super) ask: Ask,
    pub(super) input_counter: u64,
    pub(super) carrying: Option<Carrying>,
    pub(super) transfers: Arc<Vec<TransferRow>>,
    pub(super) transfer_counter: u64,
    pub(super) side: Side,
    pub(super) detail_open: bool,
    pub(super) tools_open: bool,
    pub(super) report: Option<Arc<RescanSummary>>,
    pub(super) picked: Option<PickedFile>,
}

impl FilesState {
    /// The workbench over `source`, at the top. Nothing is selected: nothing is opened by itself.
    pub fn new(source: Arc<dyn FileSource>, labels: FileLabels, variant: Variant) -> Self {
        let capabilities = match variant {
            Variant::Picker => source.capabilities().read_only(),
            _ => source.capabilities(),
        };
        let mut state = Self {
            source,
            labels: Arc::new(labels),
            capabilities,
            variant,
            view: FileView::List,
            place: Place::Folder(None),
            path: Vec::new(),
            children: HashMap::new(),
            opened: HashSet::new(),
            tree_problem: None,
            rows: Arc::new(Vec::new()),
            unreadable: Arc::new(Vec::new()),
            next: None,
            term: String::new(),
            filter: ListFilter::default(),
            filter_draft: FilterDraft::default(),
            sort: SortOrder::default(),
            list_generation: 0,
            list_problem: None,
            selected: None,
            notice: None,
            ask: Ask::Nothing,
            input_counter: 0,
            carrying: None,
            transfers: Arc::new(Vec::new()),
            transfer_counter: 0,
            side: Side::Item,
            detail_open: false,
            tools_open: false,
            report: None,
            picked: None,
        };
        state.reload_tree();
        state.reload(PAGE);
        state
    }

    /// What is current, for the host (ADR 0056).
    pub fn context(&self) -> FilesContext {
        FilesContext {
            folder: self.folder().map(str::to_string),
            in_trash: self.in_trash(),
            selected: self.selected_key().map(str::to_string),
            view: self.view,
        }
    }

    /// `name` as a question or a message names it: a marked thing says so wherever it is named.
    pub(in crate::file_manager) fn named(&self, entry: &FileEntry) -> String {
        if entry.is_marked() {
            format!("[{}] {}", self.labels.marked, entry.name)
        } else {
            entry.name.clone()
        }
    }

    /// Say what went wrong, by its kind. A taken name, a bad name and a change from outside are shown
    /// as what they are.
    pub(super) fn problem_text(&self, e: &FileError) -> String {
        let text = match e.problem {
            FileProblem::NameConflict => &self.labels.name_conflict,
            FileProblem::InvalidName => &self.labels.invalid_name,
            FileProblem::ChangedOutside => &self.labels.changed_outside,
            FileProblem::Unsupported => &self.labels.unsupported,
            _ => &self.labels.failed,
        };
        FileLabels::fill(text, &e.message)
    }

    /// Say what went wrong and read again: what is on the screen must be what is in the source.
    pub(super) fn failed(&mut self, e: &FileError) {
        let text = self.problem_text(e);
        self.after_change();
        self.notice = Some(Notice::Problem(e.problem, text));
    }
}
