//! The style sheet of all widgets of this crate.
//!
//! Pattern: none — one pure function that joins the sheets of the single widgets.

use crate::{
    agent_tree, avatar_group, breadcrumb, calendar_view, file_preview, file_tile, lightbox,
    list_view, message_composer, message_list, participant, share_dialog, sortable_list, task_card,
    text_view, tool_call, transfer_list, tree_view,
};

/// The CSS of all widgets in this crate; the shell adds it to the theme once.
pub fn styles() -> String {
    [
        text_view::STYLES,
        tree_view::STYLES,
        list_view::STYLES,
        avatar_group::STYLES,
        transfer_list::STYLES,
        sortable_list::STYLES,
        lightbox::STYLES,
        share_dialog::STYLES,
        tool_call::STYLES,
        participant::STYLES,
        message_list::STYLES,
        message_composer::STYLES,
        agent_tree::STYLES,
        calendar_view::STYLES,
        task_card::STYLES,
        [breadcrumb::STYLES, file_tile::STYLES, file_preview::STYLES]
            .concat()
            .as_str(),
    ]
    .concat()
}
