//! From a flat listing to a tree.
//!
//! Pattern: none (one pure function) — folds the paths (parts separated by `/`) into nested [`TreeNode`]s. A
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
    let folder = parts[..=depth].join("/");
    let (key, page) = (format!("{folder}/"), format!("{folder}.md"));
    if depth + 1 == parts.len() {
        // The folder of the same name (the sub-pages) was there first: the page takes its place, children kept.
        let waiting = folder.strip_suffix(".md").and_then(|stem| {
            let stem = format!("{stem}/");
            level.iter_mut().find(|n| n.key == stem)
        });
        if let Some(n) = waiting {
            n.key.clone_from(&entry.path);
            n.label.clone_from(&entry.title);
        } else {
            level.push(TreeNode::new(entry.path.clone(), entry.title.clone()));
        }
        return;
    }
    // A folder `a/` is the page `a.md` when there is one.
    let at = if let Some(i) = level.iter().position(|n| n.key == key || n.key == page) {
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
    fn a_page_and_its_sub_pages_are_one_node() {
        for list in [
            [e("handbook.md"), e("handbook/one.md"), e("notes.md")],
            [e("handbook/one.md"), e("handbook.md"), e("notes.md")],
        ] {
            let tree = tree_nodes(&list);
            assert_eq!(tree.len(), 2);
            assert_eq!(tree[0].key, "handbook.md");
            assert_eq!(tree[0].children.len(), 1);
            assert_eq!(tree[0].children[0].key, "handbook/one.md");
            assert_eq!(tree[1].key, "notes.md");
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
