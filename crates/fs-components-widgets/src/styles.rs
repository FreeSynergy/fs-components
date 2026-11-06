//! The style sheet of all widgets of this crate.
//!
//! Pattern: Composite of templates — joins the sheets of the single widgets.

use crate::{avatar_group, sortable_list, text_view, transfer_list, tree_view};

/// The CSS of all widgets in this crate; the shell adds it to the theme once.
pub fn styles() -> String {
    [
        text_view::STYLES,
        tree_view::STYLES,
        avatar_group::STYLES,
        transfer_list::STYLES,
        sortable_list::STYLES,
    ]
    .concat()
}
