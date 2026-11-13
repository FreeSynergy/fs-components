//! The page tree, lazy: built once from the flat list, drawn only as far as folders are open.
//!
//! Pattern: Flyweight-ish flat arena — nodes in pre-order with the end of their subtree, so the visible rows are one pass
//! that jumps over closed folders. With 2378 pages only the rows of open folders become elements; opening a folder
//! costs the rows of its children, not a rebuild of the whole tree.

use std::collections::HashSet;

use super::fold::fold;
use crate::source::Entry;

/// One node of the arena.
#[derive(Clone, Debug)]
pub(crate) struct Node {
    /// Path of a page (also one with sub-pages), or the path plus `/` of a folder.
    pub key: String,
    pub label: String,
    pub depth: u16,
    /// Has children (or is a folder): can be opened and closed.
    pub folder: bool,
    /// Is a page that can be read (a leaf, or a page with sub-pages); a plain folder is not.
    pub page: bool,
    /// Index after the last descendant.
    pub end: usize,
}

/// The tree and which folders are open.
#[derive(Default)]
pub(crate) struct PageTree {
    nodes: Vec<Node>,
    open: HashSet<String>,
    pages: usize,
    revealed: String,
}

/// A row to draw.
pub(crate) struct Row<'a> {
    pub node: &'a Node,
    pub open: bool,
}

impl PageTree {
    /// The tree of `entries` (path order is kept), everything closed. A page `a.md` and the folder `a/` are one node.
    pub fn new(entries: &[Entry]) -> Self {
        Self {
            nodes: fold(entries),
            open: HashSet::new(),
            pages: entries.len(),
            revealed: String::new(),
        }
    }

    /// How many pages (not folders) the tree holds.
    pub fn pages(&self) -> usize {
        self.pages
    }

    /// Open or close the folder `key`.
    pub fn toggle(&mut self, key: &str) {
        if !self.open.remove(key) {
            self.open.insert(key.to_string());
        }
    }

    /// Open every folder above the page `path`, so it can be seen.
    pub fn reveal(&mut self, path: &str) {
        let parts: Vec<&str> = path.split('/').collect();
        for depth in 0..parts.len().saturating_sub(1) {
            let folder = parts[..=depth].join("/");
            // The way may lead through a folder or through the page of the same name.
            self.open.insert(format!("{folder}.md"));
            self.open.insert(format!("{folder}/"));
        }
    }

    /// Like [`reveal`](Self::reveal), but only when `path` is not the page revealed last: a person may close the folder of the
    /// page they read.
    pub fn reveal_once(&mut self, path: &str) {
        if self.revealed != path {
            path.clone_into(&mut self.revealed);
            self.reveal(path);
        }
    }

    /// The rows to draw now: the roots, and the children of open folders. `limit` caps the rows (a closed-world guard).
    pub fn rows(&self, limit: usize) -> Vec<Row<'_>> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < self.nodes.len() && out.len() < limit {
            let node = &self.nodes[i];
            let open = node.folder && self.open.contains(&node.key);
            out.push(Row { node, open });
            i = if node.folder && !open {
                node.end
            } else {
                i + 1
            };
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(path: &str) -> Entry {
        Entry::new(path, path.rsplit('/').next().unwrap())
    }

    #[test]
    fn closed_folders_hide_their_pages() {
        let mut t = PageTree::new(&[e("a/x.md"), e("a/b/y.md"), e("z.md")]);
        let keys = |t: &PageTree| {
            t.rows(100)
                .iter()
                .map(|r| r.node.key.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(keys(&t), ["a/", "z.md"]);
        t.toggle("a/");
        assert_eq!(keys(&t), ["a/", "a/x.md", "a/b/", "z.md"]);
        t.toggle("a/b/");
        assert_eq!(keys(&t), ["a/", "a/x.md", "a/b/", "a/b/y.md", "z.md"]);
        assert_eq!(t.pages(), 3);
    }

    /// The size of the real documentation: 2378 pages in folders three deep. Building the tree and listing the rows of an
    /// opened folder must stay far below a frame (16 ms) in release; in debug the bound is loose.
    #[test]
    fn a_large_tree_is_cheap_to_build_and_to_open() {
        let entries: Vec<Entry> = (0..2378)
            .map(|i| e(&format!("docs/f{}/g{}/p{i}.md", i / 400, i / 40)))
            .collect();
        let started = std::time::Instant::now();
        let mut t = PageTree::new(&entries);
        let built = started.elapsed();
        assert_eq!(t.pages(), 2378);
        let closed = t.rows(5000).len();
        t.toggle("docs/");
        t.toggle("docs/f0/");
        let started = std::time::Instant::now();
        let opened = t.rows(5000).len();
        let listed = started.elapsed();
        eprintln!(
            "tree of 2378 pages: built in {built:?}, {closed} rows closed, {opened} rows after opening, listed in {listed:?}"
        );
        assert!(closed <= 1);
        assert!(opened < 100, "opening one folder shows its children only");
        assert!(built.as_millis() < 500 && listed.as_millis() < 100);
    }

    #[test]
    fn reveal_opens_the_way() {
        let mut t = PageTree::new(&[e("a/b/y.md"), e("z.md")]);
        t.reveal("a/b/y.md");
        assert_eq!(t.rows(100).len(), 4);
    }
}
