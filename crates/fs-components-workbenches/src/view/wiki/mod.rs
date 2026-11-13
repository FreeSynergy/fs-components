//! The wiki layouts: the same reader state in a Confluence-like frame (page tree, header, outline) or a
//! Notion-like frame (calm sidebar, big title, blocks). Patterns only; nothing is copied from either product.
//!
//! Pattern: Strategy (the layout is chosen by [`Layout`]) over the one `Reader` state; each layout is a set of
//! pure functions from state to elements.

pub(crate) mod blocks;
pub(crate) mod confluence;
mod fold;
mod head;
pub(crate) mod notion;
mod outline;
mod rows;
mod styles;
mod tree;
#[cfg(test)]
mod rows_tests;
#[cfg(test)]
mod tree_tests;

pub(crate) use blocks::BlockEditor;
pub(crate) use styles::styles;
pub(crate) use tree::PageTree;

/// Which frame the reader is drawn in.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layout {
    /// The plain reader: search and list left, page right.
    #[default]
    Classic,
    /// Page tree left, header with language switch and state, outline and history right.
    Confluence,
    /// Calm sidebar, big title, reading column, block editor.
    Notion,
}
