//! The wiki layouts: the same reader state in a Confluence-like frame (page tree, header, outline) or a
//! Notion-like frame (calm sidebar, big title, blocks). Patterns only; nothing is copied from either product.
//!
//! Pattern: Strategy (the layout is chosen by [`Layout`]) over the one `Reader` state; each layout is a set of
//! pure functions from state to elements.

mod block_contract;
mod block_enter;
#[cfg(test)]
mod block_enter_tests;
mod block_menu;
mod block_row;
mod block_text;
mod block_view;
pub(crate) mod blocks;
pub(crate) mod confluence;
mod fold;
mod head;
pub(crate) mod notion;
mod outline;
#[cfg(test)]
mod outline_tests;
mod rows;
#[cfg(test)]
mod rows_tests;
mod slash;
#[cfg(test)]
mod slash_tests;
mod styles;
mod tree;
#[cfg(test)]
mod tree_tests;

pub use blocks::BlockEditor;
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
