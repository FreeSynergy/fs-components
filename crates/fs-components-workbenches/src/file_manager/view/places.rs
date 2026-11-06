//! The smaller areas: the message line, the way to here, the tree.
//!
//! Pattern: Adapter — `breadcrumb` and `tree-view` over the state. The line with the way says that
//! changes from outside do not come in by themselves.

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::scroll;
use fs_components_widgets::TreeView;
use fs_components_widgets::breadcrumb::{Breadcrumb, Crumb};

use super::actions::FileActions;
use crate::file_manager::context::Variant;
use crate::file_manager::side::Place;
use crate::file_manager::state::{FilesState, Notice};
use crate::file_manager::tree::TOP;

pub(in crate::file_manager) fn notice_line(notice: &Notice, name: &str) -> Div {
    let class = match notice {
        Notice::Info(_) => "fs-files-notice",
        Notice::Problem(..) => "fs-files-problem",
    };
    div()
        .id(format!("{name}-notice"))
        .class(class)
        .child(text(notice.text()))
}

/// The way to where the person is. It says that changes from outside do not come in by themselves.
pub(in crate::file_manager) fn crumbs_row(
    state: &FilesState,
    name: &str,
    actions: &FileActions,
    width: f32,
) -> Div {
    let labels = state.labels();
    let mut crumbs = vec![Crumb::new(TOP, labels.root.clone())];
    if state.place() == &Place::Trash {
        crumbs.push(Crumb::new("\u{0}trash", labels.trash_place.clone()));
    }
    crumbs.extend(
        state
            .path()
            .iter()
            .map(|f| Crumb::new(f.key.clone(), f.name.clone())),
    );
    let wide = width >= 960.0 && state.capabilities().rescan;
    let room = if wide { width * 0.6 } else { width - 24.0 };
    let mut row = div().flex_row().items_center().w_full().child(
        Breadcrumb::new(crumbs, name.to_string())
            .on_choose(actions.on_key(|s, key| {
                s.open_folder((key != TOP).then_some(key));
            }))
            .build(room),
    );
    if wide {
        row = row.child(
            div()
                .class("fs-files-outside")
                .child(text(&labels.outside_hint).no_wrap()),
        );
    }
    row
}

pub(in crate::file_manager) fn tree_pane(state: &FilesState, actions: &FileActions) -> Div {
    let mut pane = div().flex_col().w_full().h_full();
    if let Some(problem) = state.tree_problem() {
        pane = pane.child(div().class("fs-files-failed-line").child(text(problem)));
    }
    if state.variant() == Variant::Picker {
        return pane;
    }
    let choose = actions.on_key(FilesState::choose_in_tree);
    let current = state.folder().unwrap_or(TOP).to_string();
    let tree = TreeView::new(state.tree_nodes())
        .selected(current)
        .on_select(move |key| choose(key))
        .build();
    pane.child(scroll().w_full().h_full().content(tree))
}
