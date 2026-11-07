//! `tree-view`: a tree of labels you can open, close and select.
//!
//! Pattern: Adapter over `blinc_cn`'s tree — [`TreeNode`] is our neutral description, [`TreeView`]
//! builds the Blinc tree from it. Keyboard (arrows, Home, End, Enter) comes with the Blinc tree.

mod node;
mod styles;
mod view;

pub use node::TreeNode;
pub use styles::STYLES;
pub use view::TreeView;
