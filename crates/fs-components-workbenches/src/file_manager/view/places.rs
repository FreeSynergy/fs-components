//! The smaller areas: the message line, the way to here, the tree.
//!
//! Pattern: Adapter — `breadcrumb` and `tree-view` over the state. The line with the way says that
//! changes from outside do not come in by themselves.

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::scroll;
use fs_components_widgets::TreeView;
use fs_components_widgets::icon::{Glyph, Tone, icon};
use fs_components_widgets::breadcrumb::{Breadcrumb, Crumb};

use super::actions::FileActions;
use crate::file_manager::context::Variant;
use crate::file_manager::labels::FileLabels;
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
/// The way to where the person is, as a breadcrumb that fits `room`.
pub(in crate::file_manager) fn crumbs(
    state: &FilesState,
    name: &str,
    actions: &FileActions,
    room: f32,
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
    div().flex_row().items_center().w_full().child(
        Breadcrumb::new(crumbs, name.to_string())
            .on_choose(actions.on_key(|s, key| {
                s.open_folder((key != TOP).then_some(key));
            }))
            .build(room),
    )
}

/// The line at the bottom: how many things are here, and what is selected.
pub(in crate::file_manager) fn status_line(state: &FilesState) -> Div {
    let labels = state.labels();
    let mut line = div().flex_row().items_center().gap(4.0).w_full().child(
        text(FileLabels::fill(
            &labels.status_items,
            &state.rows().len().to_string(),
        ))
        .no_cursor(),
    );
    if let Some(entry) = state.selected_entry() {
        let mut said = FileLabels::fill(&labels.status_selected, "1");
        if entry.kind == crate::file_manager::model::EntryKind::File {
            said = format!(
                "{said}, {}",
                fs_components_widgets::transfer_list::format_bytes(entry.size)
            );
        }
        line = line.child(text(said).no_cursor());
    }
    line
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
    let places = places_group(state, actions);
    pane.child(places)
        .child(
            div()
                .w_full()
                .flex_grow()
                .min_h(0.0)
                .child(scroll().w_full().h_full().content(tree)),
        )
}

/// The places, in groups: where the files are (the own storage), and the trash. A place that the source
/// does not have is not shown.
fn places_group(state: &FilesState, actions: &FileActions) -> Div {
    let labels = state.labels();
    let in_trash = state.in_trash();
    let row = |id: &str, glyph: Glyph, word: &str, current: bool, act: super::actions::Act| {
        let one = div().id(id.to_string()).class("fs-files-place");
        let one = if current {
            one.class("fs-files-place-current")
        } else {
            one
        };
        one.w_full()
            .flex_row()
            .justify_start()
            .items_center()
            .gap(2.0)
            .attr("tabindex", "0")
            .on_click(move |_| act())
            .child(icon(glyph, 18.0, if current { Tone::Accent } else { Tone::Kind }))
            .child(text(word).no_cursor())
    };
    let title = |word: &str| {
        div()
            .class("fs-files-section-title")
            .child(text(word).no_cursor())
    };
    let mut group = div()
        .flex_col()
        .w_full()
        .flex_shrink_0()
        .child(title(&labels.places))
        .child(row(
            "fs-files-place-top",
            Glyph::Home,
            &labels.root,
            !in_trash && state.folder().is_none(),
            actions.with(|s| s.open_folder(None)),
        ));
    if state.capabilities().trash {
        group = group.child(row(
            "fs-files-place-trash",
            Glyph::Trash,
            &labels.trash,
            in_trash,
            actions.plain(FilesState::open_trash),
        ));
    }
    group.child(title(&labels.folders_title))
}
