//! The folder tree: what is known of it, and which folders are open.
//!
//! Pattern: State + Composite — the children of a folder are read when it is opened, and at most
//! [`TREE_LIMIT`] of them, because Blinc's tree is built whole. The way to the current folder is open.
//! A folder cannot be opened by hand without going there: `tree-view` reports only a choice.

use fs_components_widgets::TreeNode;

use super::error::FileError;
use super::labels::FileLabels;
use super::model::FolderInfo;
use super::side::Place;
use super::state::{FilesState, Notice};

/// Folders read per folder for the tree.
pub const TREE_LIMIT: usize = 200;
/// Deepest folder followed upwards to the top.
const DEEPEST: usize = 512;
/// The key of the top in the tree.
pub(super) const TOP: &str = "";
/// The start of the key of a line that says there are more.
const MORE: &str = "\u{0}more:";

impl FilesState {
    /// The folders from the top to `key`; `None` if it or one on the way is gone.
    pub(super) fn path_to(&self, key: &str) -> Result<Option<Vec<FolderInfo>>, FileError> {
        let mut path = Vec::new();
        let mut at = Some(key.to_string());
        while let Some(current) = at.take() {
            let Some(folder) = self.source.folder(&current)? else {
                return Ok(None);
            };
            at.clone_from(&folder.parent);
            path.push(folder);
            if path.len() > DEEPEST {
                return Err(FileError::failed("the folders go too deep"));
            }
        }
        path.reverse();
        Ok(Some(path))
    }

    /// The way to `key` is open and known.
    pub(super) fn open_tree_to(&mut self, key: Option<&str>) {
        let keys: Vec<String> = self.path.iter().map(|f| f.key.clone()).collect();
        self.opened.extend(keys);
        if let Some(key) = key {
            self.opened.insert(key.to_string());
        }
        self.reload_tree();
    }

    /// Read again what is known of the tree; a failure keeps it and says why.
    pub(super) fn reload_tree(&mut self) {
        if self.variant == super::context::Variant::Picker {
            return;
        }
        let mut wanted: Vec<Option<String>> = vec![None];
        wanted.extend(self.opened.iter().cloned().map(Some));
        self.tree_problem = None;
        let mut gone = Vec::new();
        for key in wanted {
            match self.source.subfolders(key.as_deref(), TREE_LIMIT) {
                Ok(children) => {
                    self.children.insert(key, children);
                }
                Err(e) if e.problem == super::error::FileProblem::NotFound => {
                    if let Some(key) = key {
                        self.children.remove(&Some(key.clone()));
                        gone.push(key);
                    }
                }
                Err(e) => {
                    self.tree_problem = Some(FileLabels::fill(
                        &self.labels.tree_failed,
                        &self.problem_text(&e),
                    ));
                }
            }
        }
        for key in gone {
            self.opened.remove(&key);
        }
        self.leave_folder_if_gone();
    }

    /// The current folder may be gone: then the top is shown, and that is said.
    fn leave_folder_if_gone(&mut self) {
        let Place::Folder(Some(key)) = &self.place else {
            return;
        };
        let key = key.clone();
        match self.path_to(&key) {
            Ok(Some(path)) => self.path = path,
            Ok(None) => {
                self.place = Place::Folder(None);
                self.path.clear();
                self.list_generation += 1;
                self.reload(super::state::PAGE);
                self.notice = Some(Notice::Info(self.labels.folder_gone.clone()));
            }
            Err(_) => {}
        }
    }

    /// Why the tree could not be read, if it could not.
    pub fn tree_problem(&self) -> Option<&str> {
        self.tree_problem.as_deref()
    }

    /// The tree as far as it is known. The key of a folder is its own; the top is the empty text.
    pub fn tree_nodes(&self) -> Vec<TreeNode> {
        let mut top = TreeNode::new(TOP, self.labels.root.clone());
        top.open = true;
        top.children = self.nodes_in(None, DEEPEST);
        vec![top]
    }

    fn nodes_in(&self, parent: Option<&str>, depth: usize) -> Vec<TreeNode> {
        let Some((children, more)) = self.children.get(&parent.map(str::to_string)) else {
            return Vec::new();
        };
        let mut nodes = Vec::with_capacity(children.len());
        for folder in children {
            let mut node = TreeNode::new(folder.key.clone(), folder.name.clone());
            if depth > 0 && self.opened.contains(&folder.key) {
                node.open = true;
                node.children = self.nodes_in(Some(&folder.key), depth - 1);
            }
            nodes.push(node);
        }
        if *more {
            nodes.push(TreeNode::new(
                format!("{MORE}{}", parent.unwrap_or(TOP)),
                self.labels.tree_more.clone(),
            ));
        }
        nodes
    }

    /// A line of the tree was chosen. The line that says there are more leads to their parent; the
    /// folder the person is in closes or opens.
    pub fn choose_in_tree(&mut self, key: &str) {
        let key = key.strip_prefix(MORE).unwrap_or(key);
        let target = (key != TOP).then_some(key);
        if !self.in_trash() && self.folder() == target {
            if let Some(key) = target
                && !self.opened.remove(key)
            {
                self.opened.insert(key.to_string());
            }
            return;
        }
        self.open_folder(target);
    }
}
