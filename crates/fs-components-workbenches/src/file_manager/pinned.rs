//! The pinned folders: the user folders of the desktop (Desktop, Downloads, Documents, Pictures, Music, Videos) that
//! the storage has, shown at the side.
//!
//! Pattern: Strategy by data — the host says by which NAME it knows each kind (the names come from the user folders
//! of the system, in the language of the person: `Bilder`, `Pictures`); the window pins the folders at the top of its
//! storage that have such a name. A kind with no folder is not shown: nothing is made up.

use fs_components_widgets::PinnedKind;

use super::model::FolderInfo;
use super::state::FilesState;

impl FilesState {
    /// The names by which the folders to pin are known; replaces what was given before.
    pub fn pin_by_names(&mut self, names: Vec<(PinnedKind, String)>) {
        self.pinned_names = names;
    }

    /// The folders at the top of the storage that are pinned, with their kind, in the order the host gave the names.
    pub fn pinned(&self) -> Vec<(PinnedKind, FolderInfo)> {
        let Some((top, _)) = self.children.get(&None) else {
            return Vec::new();
        };
        self.pinned_names
            .iter()
            .filter_map(|(kind, name)| {
                top.iter()
                    .find(|f| f.name.eq_ignore_ascii_case(name))
                    .map(|f| (*kind, f.clone()))
            })
            .collect()
    }
}
