//! The style sheet of all widgets of this crate.
//!
//! Pattern: Composite of templates — joins the sheets of the single widgets.

use crate::{text_view, tree_view};

/// The CSS of all widgets in this crate; the shell adds it to the theme once.
pub fn styles() -> String {
    [text_view::STYLES, tree_view::STYLES].concat()
}
