//! A node of the tree.
//!
//! Pattern: Composite — a node holds nodes.

/// One node: a key the caller understands, a label to show, and children.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TreeNode {
    /// Identity given back on selection.
    pub key: String,
    /// What is shown.
    pub label: String,
    /// Shown open?
    pub open: bool,
    /// Children.
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    /// A leaf.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            open: false,
            children: Vec::new(),
        }
    }

    /// Number of nodes in this tree, itself included.
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(TreeNode::count).sum::<usize>()
    }
}
