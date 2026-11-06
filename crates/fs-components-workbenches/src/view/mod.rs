//! The look of the reader: pure functions from the state to elements.
//!
//! Pattern: Adapter (state to Blinc elements) — each function draws one area from a `&Reader`
//! and reports what the person does through [`Actions`]; nothing here changes state itself.

mod actions;
mod content;
mod frame;
mod results;
mod sidebar;
mod styles;
mod tree_build;

pub(crate) use actions::Actions;
pub(crate) use content::content;
pub(crate) use frame::frame;
pub(crate) use sidebar::{search_box, side_body};
pub(crate) use styles::styles;
pub(crate) use tree_build::tree_nodes;
