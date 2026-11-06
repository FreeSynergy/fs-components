//! Going to folders, the list and what decides it: search, sort, filter.
//!
//! Pattern: State — every way to another list counts up its generation (its scroll position is void).
//! A list that cannot be read keeps what is drawn and says why, and can be tried again. While something
//! is typed or asked, going elsewhere does not lose it: an open input stays open.

use std::sync::Arc;

use super::ask::Ask;
use super::labels::FileLabels;
use super::model::{ListRequest, Scope, SortKey, SortOrder};
use super::side::{Place, Side};
use super::state::{FilesState, Notice, PAGE};

impl FilesState {
    /// Go to this folder (`None` = the top). A folder that is gone is said so, and the list stays.
    pub fn open_folder(&mut self, key: Option<&str>) {
        let path = match key {
            Some(key) => match self.path_to(key) {
                Ok(Some(path)) => path,
                Ok(None) => {
                    self.notice = Some(Notice::Info(self.labels.folder_gone.clone()));
                    self.reload_tree();
                    return;
                }
                Err(e) => {
                    self.failed(&e);
                    return;
                }
            },
            None => Vec::new(),
        };
        self.leave_for(Place::Folder(key.map(str::to_string)));
        self.path = path;
        self.open_tree_to(key);
    }

    /// Show the trash. A move in progress goes on: a restore to a place chooses its place out here.
    pub fn open_trash(&mut self) {
        if self.capabilities.trash {
            self.path.clear();
            self.leave_for(Place::Trash);
        }
    }

    /// A new list in another place, from its top, with nothing selected.
    fn leave_for(&mut self, place: Place) {
        self.place = place;
        self.selected = None;
        if matches!(self.ask, Ask::Confirm(_) | Ask::NameTaken(_)) {
            self.ask = Ask::Nothing;
        }
        self.detail_open = false;
        self.side = Side::Item;
        self.term.clear();
        self.new_list();
    }

    fn new_list(&mut self) {
        self.list_generation += 1;
        self.rows = Arc::new(Vec::new());
        self.unreadable = Arc::new(Vec::new());
        self.next = None;
        self.reload(PAGE);
    }

    /// Search: in all folders, or in the trash while that is shown. It does not leave the thing shown.
    pub fn search(&mut self, term: &str) {
        let term = term.trim();
        if term == self.term || !self.capabilities.search {
            return;
        }
        self.term = term.to_string();
        self.new_list();
    }

    /// Sort by this. By type it can only go forwards if the source says so.
    pub fn set_sort(&mut self, key: SortKey) {
        let descending =
            self.sort.descending && (key != SortKey::Type || self.capabilities.type_descending);
        self.sort = SortOrder { key, descending };
        self.new_list();
    }

    /// The other direction, if there is one for this order.
    pub fn toggle_direction(&mut self) {
        if self.can_toggle_direction() {
            self.sort.descending = !self.sort.descending;
            self.new_list();
        }
    }

    /// Can the order be turned around (show the button)?
    pub fn can_toggle_direction(&self) -> bool {
        self.sort.key != SortKey::Type || self.capabilities.type_descending
    }

    pub(super) fn filter_changed(&mut self) {
        self.new_list();
    }

    fn request(&self) -> ListRequest {
        let scope = match &self.place {
            Place::Trash => Scope::Trash,
            Place::Folder(_) if !self.term.is_empty() => Scope::Everywhere,
            Place::Folder(folder) => Scope::Folder(folder.clone()),
        };
        ListRequest {
            scope,
            term: self.term.clone(),
            filter: self.filter.clone(),
            sort: self.sort,
        }
    }

    /// Read the list again, at least `count` rows. A failure keeps what is drawn, says why, and a retry
    /// is offered.
    pub(super) fn reload(&mut self, count: usize) {
        let request = self.request();
        let mut rows = Vec::new();
        let mut unreadable = Vec::new();
        let mut after = None;
        loop {
            match self.source.list(&request, after.as_ref()) {
                Ok(page) => {
                    rows.extend(page.entries);
                    unreadable.extend(page.unreadable);
                    after = page.next;
                    if after.is_none() || rows.len() >= count {
                        break;
                    }
                }
                Err(e) => {
                    self.list_problem = Some(FileLabels::fill(
                        &self.labels.list_failed,
                        &self.problem_text(&e),
                    ));
                    return;
                }
            }
        }
        self.rows = Arc::new(rows);
        self.unreadable = Arc::new(unreadable);
        self.next = after;
        self.list_problem = None;
    }

    /// Load the next rows. A failure keeps the rows, says why, and can be tried again.
    pub fn load_more(&mut self) {
        let request = self.request();
        let mut rows = self.rows.as_ref().clone();
        let mut unreadable = self.unreadable.as_ref().clone();
        let goal = rows.len() + PAGE;
        while let Some(mark) = self.next.clone() {
            match self.source.list(&request, Some(&mark)) {
                Ok(page) => {
                    rows.extend(page.entries);
                    unreadable.extend(page.unreadable);
                    self.next = page.next;
                    self.list_problem = None;
                }
                Err(e) => {
                    self.list_problem = Some(FileLabels::fill(
                        &self.labels.list_failed,
                        &self.problem_text(&e),
                    ));
                    break;
                }
            }
            if rows.len() >= goal {
                break;
            }
        }
        self.rows = Arc::new(rows);
        self.unreadable = Arc::new(unreadable);
    }

    /// Read the list again, after it could not be read or at the person's wish.
    pub fn retry(&mut self) {
        let count = self.rows.len().max(PAGE);
        self.reload(count);
        self.reload_tree();
    }

    /// After a change: the list, the tree and the thing selected are read again; what is gone leaves.
    pub(super) fn after_change(&mut self) {
        let count = self.rows.len().max(PAGE);
        self.reload(count);
        self.reload_tree();
        self.refresh_selected();
    }
}
