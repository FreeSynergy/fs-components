//! What the frame of the window remembers: the menu of "New", the detail area, folded groups, the next order.
//!
//! Pattern: State — small switches of the interface, none of them touches the source.

use super::model::SortKey;
use super::state::FilesState;

impl FilesState {
    /// Is the menu of the "New" button open?
    pub fn new_menu_open(&self) -> bool {
        self.new_menu
    }

    /// Open or close the menu of the "New" button.
    pub fn toggle_new_menu(&mut self) {
        self.new_menu = !self.new_menu;
    }

    /// Is the detail area allowed to show what is selected?
    pub fn details_on(&self) -> bool {
        !self.details_off
    }

    /// Show or hide the detail area.
    pub fn toggle_details(&mut self) {
        self.details_off = !self.details_off;
    }

    /// Is this group of the sidebar folded away?
    pub fn group_folded(&self, group: &str) -> bool {
        self.collapsed.contains(group)
    }

    /// Fold a group of the sidebar away, or open it again.
    pub fn toggle_group(&mut self, group: &str) {
        if !self.collapsed.remove(group) {
            self.collapsed.insert(group.to_string());
        }
    }

    /// The next order of the list: name, modified, size, type, and round again.
    pub fn next_sort(&mut self) {
        let next = match self.sort.key {
            SortKey::Name => SortKey::Modified,
            SortKey::Modified => SortKey::Size,
            SortKey::Size => SortKey::Type,
            _ => SortKey::Name,
        };
        self.set_sort(next);
    }
}
