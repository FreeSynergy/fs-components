//! What the view and a host may read of the state.
//!
//! Pattern: State (read side) — plain getters, and the small switches of the view.

use std::sync::Arc;

use super::ask::Ask;
use super::carry::Carrying;
use super::context::{FileView, Variant};
use super::labels::FileLabels;
use super::model::{FileEntry, FolderInfo, ListFilter, SortOrder};
use super::reports::FileCapabilities;
use super::side::{Place, Side};
use super::state::{FilesState, Notice};

impl FilesState {
    /// The words.
    pub fn labels(&self) -> &FileLabels {
        &self.labels
    }

    /// Full or picker.
    pub fn variant(&self) -> Variant {
        self.variant
    }

    /// What the source can do, as far as this variant offers it.
    pub fn capabilities(&self) -> FileCapabilities {
        self.capabilities
    }

    /// The view shown.
    pub fn view(&self) -> FileView {
        self.view
    }

    /// Show the entries this way.
    pub fn set_view(&mut self, view: FileView) {
        self.view = view;
    }

    /// Where the person is.
    pub fn place(&self) -> &Place {
        &self.place
    }

    /// The current folder; `None` = the top, also while the trash is shown.
    pub fn folder(&self) -> Option<&str> {
        match &self.place {
            Place::Folder(folder) => folder.as_deref(),
            _ => None,
        }
    }

    /// Is the trash shown?
    pub fn in_trash(&self) -> bool {
        self.place == Place::Trash
    }

    /// The folders from the top down to the current one (without the top itself).
    pub fn path(&self) -> &[FolderInfo] {
        &self.path
    }

    /// The rows loaded so far.
    pub fn rows(&self) -> &Arc<Vec<FileEntry>> {
        &self.rows
    }

    /// What could not be read of the list, by name.
    pub fn unreadable(&self) -> &[String] {
        &self.unreadable
    }

    /// Are there more rows than are loaded?
    pub fn has_more(&self) -> bool {
        self.next.is_some()
    }

    /// The search text.
    pub fn term(&self) -> &str {
        &self.term
    }

    /// The order.
    pub fn sort(&self) -> SortOrder {
        self.sort
    }

    /// The filter in force.
    pub fn filter(&self) -> &ListFilter {
        &self.filter
    }

    /// Counts up whenever another list takes the place of this one: its scroll position is void.
    pub fn list_generation(&self) -> u64 {
        self.list_generation
    }

    /// Why the list could not be read, if it could not; the rows drawn stay.
    pub fn list_problem(&self) -> Option<&str> {
        self.list_problem.as_deref()
    }

    /// A short message for the person, if any.
    pub fn notice(&self) -> Option<&Notice> {
        self.notice.as_ref()
    }

    /// What the workbench is waiting for.
    pub fn ask(&self) -> &Ask {
        &self.ask
    }

    /// What is being moved or copied, if anything.
    pub fn carrying(&self) -> Option<&Carrying> {
        self.carrying.as_ref()
    }

    /// What the side area shows.
    pub fn side(&self) -> Side {
        self.side
    }

    /// At narrow widths the side area takes the place of the list while this is set.
    pub fn detail_open(&self) -> bool {
        self.detail_open
    }

    /// Go back to the list from the side area, at narrow widths.
    pub fn close_detail(&mut self) {
        self.detail_open = false;
    }

    /// Are the tools shown at narrow widths?
    pub fn tools_open(&self) -> bool {
        self.tools_open
    }

    /// Show or hide the tools at narrow widths.
    pub fn toggle_tools(&mut self) {
        self.tools_open = !self.tools_open;
    }
}
