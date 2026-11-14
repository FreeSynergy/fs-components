//! The look of the reader: pure functions from the state to elements.
//!
//! Pattern: Adapter (state to Blinc elements) — each function draws one area from a `&Reader`
//! and reports what the person does through [`Actions`]; nothing here changes state itself.

mod actions;
mod content;
mod editor;
mod frame;
mod results;
mod sidebar;
mod source_editor;
mod styles;
mod tree_build;
mod wiki;

pub(crate) use actions::Actions;
pub(crate) use content::content;
pub(crate) use editor::{editor, editor_rows, measure_again, new_area};
pub(crate) use frame::frame;
pub(crate) use results::results as results_list;
pub(crate) use sidebar::{search_box, side_body};
pub use source_editor::SourceEditor;
pub(crate) use styles::styles;
pub(crate) use tree_build::tree_nodes;
pub use wiki::BlockEditor;
pub use wiki::Layout;
pub(crate) use wiki::PageTree;
pub(crate) use wiki::{confluence, notion};
