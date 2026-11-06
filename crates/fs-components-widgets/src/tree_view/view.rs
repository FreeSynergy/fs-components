//! The tree view itself.
//!
//! Pattern: Adapter — our [`TreeNode`]s in, `blinc_cn`'s tree out.

use blinc_cn::components::tree::{TreeNodeConfig, tree_view};

use super::node::TreeNode;
use crate::Element;

/// Called with the key of the selected node.
type OnSelect = Box<dyn Fn(&str) + Send + Sync>;

/// A tree of [`TreeNode`]s.
pub struct TreeView {
    roots: Vec<TreeNode>,
    selected: Option<String>,
    on_select: Option<OnSelect>,
}

impl TreeView {
    /// A tree with these top-level nodes.
    pub fn new(roots: Vec<TreeNode>) -> Self {
        Self {
            roots,
            selected: None,
            on_select: None,
        }
    }

    /// Start with `key` selected.
    #[must_use]
    pub fn selected(mut self, key: impl Into<String>) -> Self {
        self.selected = Some(key.into());
        self
    }

    /// Call `handler` with the key of the node the person selects (click, Enter, Space).
    #[must_use]
    pub fn on_select(mut self, handler: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }

    /// The element to place in a layout.
    pub fn build(self) -> impl Element {
        let mut tree = tree_view();
        for root in &self.roots {
            tree = tree.add_node(config(root));
        }
        if let Some(key) = self.selected {
            tree = tree.selected(key);
        }
        if let Some(handler) = self.on_select {
            tree = tree.on_select(handler);
        }
        tree.class("fs-tree")
    }
}

/// Our node as `blinc_cn`'s.
fn config(node: &TreeNode) -> TreeNodeConfig {
    let mut out = TreeNodeConfig::new(node.key.clone(), node.label.clone());
    if node.open {
        out = out.expanded();
    }
    out.children = node.children.iter().map(config).collect();
    out
}
