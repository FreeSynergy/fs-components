//! `fs-components-text-editor`: the text editor of the wiki as a component.
//!
//! Markdown source with a toolbar (bold, italic, heading, list, link, code, quote), undo and redo,
//! counters, and a preview drawn by [`fs_components_widgets::TextView`]. Three layers: the pure
//! [`model`] (no window, tested without one), the trait [`TextEditor`] a program depends on, and
//! the view ([`EditorView`]) that reports every change to a listener the caller hands in.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

mod editor;
mod labels;
pub mod model;
mod view;

pub use editor::TextEditor;
pub use labels::Labels;
pub use model::{Counts, Edit, EditCommand, EditorModel, Selection};
pub use view::{Change, EditorView, OnChange, Previewer};
