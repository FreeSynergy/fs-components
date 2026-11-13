//! The style sheet of all widgets of this crate.
//!
//! Pattern: none — one pure function that joins the sheets of the single widgets.

use crate::{
    agent_tree, avatar_group, badge, breadcrumb, calendar_view, file_preview, file_tile, lightbox,
    list_view, message_composer, message_list, name_dialog, participant, plan, share_dialog, sortable_list,
    task_card, text_view, tool_call, transfer_list, tree_view,
};

/// The CSS of all widgets in this crate; the shell adds it to the theme once.
pub fn styles() -> String {
    [
        fs_components_styles::styles::craft_sheet::SHEET,
        text_view::STYLES,
        tree_view::STYLES,
        list_view::STYLES,
        avatar_group::STYLES,
        badge::STYLES,
        fs_components_primitives::SEARCH_FIELD_STYLES,
        transfer_list::STYLES,
        sortable_list::STYLES,
        lightbox::STYLES,
        share_dialog::STYLES,
        name_dialog::STYLES,
        tool_call::STYLES,
        participant::STYLES,
        message_list::STYLES,
        message_composer::STYLES,
        agent_tree::STYLES,
        calendar_view::STYLES,
        task_card::STYLES,
        plan::STYLES,
        [breadcrumb::STYLES, file_tile::STYLES, file_preview::STYLES]
            .concat()
            .as_str(),
    ]
    .concat()
}
