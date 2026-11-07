//! The sidebar: a search box above either the tree of documents or the search results.
//!
//! Pattern: State — the body shows results while there is a search text, the tree
//! otherwise.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;
use blinc_layout::widgets::scroll::scroll;
use fs_components_primitives::SearchField;
use fs_components_widgets::{TreeNode, TreeView};

use super::actions::Actions;
use super::results::results;
use crate::reader::Reader;

/// The search box.
pub(crate) fn search_box(field: &SearchField) -> impl ElementBuilder + use<> {
    field.build()
}

/// The body below the search box.
pub(crate) fn side_body(reader: &Reader, tree: &[TreeNode], actions: &Actions) -> Div {
    let inner = if reader.term().is_empty() && reader.problem().is_some() {
        results(reader, actions)
    } else if reader.term().is_empty() {
        let open = actions.open.clone();
        let mut view = TreeView::new(tree.to_vec()).on_select(move |key| {
            if !key.ends_with('/') {
                open(key);
            }
        });
        if let Some(path) = reader.current_path() {
            view = view.selected(path);
        }
        div().child(view.build())
    } else {
        results(reader, actions)
    };
    div()
        .class("fs-rd-side-body")
        .flex_1()
        .min_h(0.0)
        .child(scroll().w_full().h_full().content(inner))
}
