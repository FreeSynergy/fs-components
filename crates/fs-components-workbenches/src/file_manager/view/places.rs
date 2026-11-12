//! The smaller areas: the message line, the way to here, the status line, the sidebar.
//!
//! Pattern: Adapter — `breadcrumb` and `tree-view` over the state. The line with the way says that
//! changes from outside do not come in by themselves.

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::breadcrumb::{Breadcrumb, Crumb};
use fs_components_widgets::icon::{Glyph, Tone, icon};

use super::actions::FileActions;
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
    let mut said = FileLabels::fill(&labels.status_items, &state.rows().len().to_string());
    if let Some(entry) = state.selected_entry() {
        said = format!(
            "{said}  |  {}",
            FileLabels::fill(&labels.status_selected, "1")
        );
        if entry.kind == crate::file_manager::model::EntryKind::File {
            said = format!(
                "{said}, {}",
                fs_components_widgets::transfer_list::format_bytes(entry.size)
            );
        }
    }
    div()
        .flex_row()
        .items_center()
        .w_full()
        .child(text(said).no_cursor())
}

/// A group of the sidebar: its title with the arrow that folds it, and what it holds while it is open.
pub(super) fn group(
    into: Div,
    state: &FilesState,
    actions: &FileActions,
    (key, title): (&'static str, &str),
    rows: impl FnOnce(Div) -> Div,
) -> Div {
    let folded = state.group_folded(key);
    let toggle = actions.on_key(FilesState::toggle_group);
    let header = div()
        .id(format!("fs-files-group-{key}"))
        .class("fs-files-section-title")
        .flex_row()
        .items_center()
        .gap(1.0)
        .w_full()
        .attr("tabindex", "0")
        .on_click(move |_| toggle(key))
        .child(icon(
            if folded {
                Glyph::ChevronRight
            } else {
                Glyph::Down
            },
            12.0,
            Tone::Muted,
        ))
        .child(text(title).no_cursor());
    let mut inner = div().flex_col().w_full().child(header);
    if !folded {
        inner = rows(inner);
    }
    into.child(inner)
}

pub(super) fn place_row(
    id: &str,
    glyph: Glyph,
    word: &str,
    current: bool,
    act: super::actions::Act,
) -> Div {
    let one = div().id(id.to_string()).class("fs-files-place");
    let one = if current {
        one.class("fs-files-place-current")
    } else {
        one
    };
    one.w_full()
        .h(super::sidebar::ROW)
        .flex_row()
        .justify_start()
        .items_center()
        .gap(2.0)
        .attr("tabindex", "0")
        .on_click(move |_| act())
        .child(icon(
            glyph,
            18.0,
            if current { Tone::Accent } else { Tone::Kind },
        ))
        .child(text(word).no_cursor())
}
