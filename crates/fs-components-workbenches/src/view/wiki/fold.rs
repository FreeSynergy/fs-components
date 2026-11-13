//! From the flat list to the arena: folders from path parts, a page `a.md` and the folder `a/` as ONE node.
//!
//! Pattern: Composite (a page may have children) — the sub-pages of `handbook.md` live under `handbook/`, so both
//! names fold into the one node of the page, which can be opened (its title) and expanded (its arrow).

use super::tree::Node;
use crate::source::Entry;

/// A node while the tree is still nested.
struct Tmp {
    node: Node,
    /// The folder name (or the page name without `.md`) the node answers to; none for other files.
    name: Option<String>,
    kids: Vec<Tmp>,
}

/// The arena (pre-order, `end` after the last descendant) of `entries`; path order is kept.
pub(super) fn fold(entries: &[Entry]) -> Vec<Node> {
    let mut roots: Vec<Tmp> = Vec::new();
    for entry in entries {
        let parts: Vec<&str> = entry.path.split('/').collect();
        insert(&mut roots, &parts, 0, entry);
    }
    let mut out = Vec::with_capacity(entries.len() + entries.len() / 4);
    flatten(roots, 0, &mut out);
    out
}

fn insert(level: &mut Vec<Tmp>, parts: &[&str], depth: usize, entry: &Entry) {
    let part = parts[depth];
    if depth + 1 == parts.len() {
        let stem = part.strip_suffix(".md");
        let mut node = Node {
            key: entry.path.clone(),
            label: entry.title.clone(),
            depth: 0,
            folder: false,
            page: true,
            end: 0,
        };
        // The folder of the same name was there first: the page takes its place.
        let waiting = stem.and_then(|s| {
            level
                .iter_mut()
                .find(|t| t.name.as_deref() == Some(s) && !t.node.page)
        });
        if let Some(t) = waiting {
            node.folder = true;
            t.node = node;
        } else {
            level.push(Tmp {
                node,
                name: stem.map(str::to_string),
                kids: Vec::new(),
            });
        }
        return;
    }
    let at = if let Some(i) = level.iter().position(|t| t.name.as_deref() == Some(part)) {
        i
    } else {
        level.push(Tmp {
            node: Node {
                key: format!("{}/", parts[..=depth].join("/")),
                label: part.to_string(),
                depth: 0,
                folder: true,
                page: false,
                end: 0,
            },
            name: Some(part.to_string()),
            kids: Vec::new(),
        });
        level.len() - 1
    };
    level[at].node.folder = true;
    insert(&mut level[at].kids, parts, depth + 1, entry);
}

fn flatten(level: Vec<Tmp>, depth: u16, out: &mut Vec<Node>) {
    for t in level {
        let at = out.len();
        out.push(Node {
            depth,
            end: at + 1,
            ..t.node
        });
        flatten(t.kids, depth.saturating_add(1), out);
        out[at].end = out.len();
    }
}
