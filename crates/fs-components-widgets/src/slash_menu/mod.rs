//! `slash-menu`: the words of the block menu of the Notion-like editor ("/" in an empty block).
//!
//! Pattern: Factory method over `MessageFormat` 2 files (ADR 0057), like the page title; the menu itself is drawn by the
//! block editor of the workbenches, which knows the blocks.

mod labels;

pub use labels::{SlashLabels, TEXTS};
