//! From a flat listing to a tree.
//!
//! Pattern: Builder — folds the paths (parts separated by `/`) into nested [`TreeNode`]s. A
//! document's key is its path; a folder's key is its path plus a trailing `/`, which no path has.

use fs_components_widgets::TreeNode;

use crate::source::Entry;

/// Folders and documents of `entries` (path order is kept).
pub(crate) fn tree_nodes(entries: &[Entry]) -> Vec<TreeNode> {
    let mut roots: Vec<TreeNode> = Vec::new();
    for entry in entries {
        let parts: Vec<&str> = entry.path.split('/').collect();
        insert(&mut roots, &parts, 0, entry);
    }
    roots
}

fn insert(level: &mut Vec<TreeNode>, parts: &[&str], depth: usize, entry: &Entry) {
    if depth + 1 == parts.len() {
        level.push(TreeNode::new(entry.path.clone(), entry.title.clone()));
        return;
    }
    let key = format!("{}/", parts[..=depth].join("/"));
    let at = if let Some(i) = level.iter().position(|n| n.key == key) {
        i
    } else {
        level.push(TreeNode::new(key, parts[depth]));
        level.len() - 1
    };
    insert(&mut level[at].children, parts, depth + 1, entry);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(path: &str) -> Entry {
        Entry {
            path: path.into(),
            title: path.rsplit('/').next().unwrap().into(),
        }
    }

    #[test]
    fn folders_group_their_documents() {
        let tree = tree_nodes(&[e("a/x.md"), e("a/b/y.md"), e("z.md")]);
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].key, "a/");
        assert_eq!(tree[0].children.len(), 2);
        assert_eq!(tree[0].children[1].key, "a/b/");
        assert_eq!(tree[0].children[1].children[0].key, "a/b/y.md");
        assert_eq!(tree[1].key, "z.md");
        assert_eq!(tree.iter().map(TreeNode::count).sum::<usize>(), 5);
    }
}
