//! The list or the grid, and what stands around it: why it could not be read, what could not be shown,
//! Show more.
//!
//! Pattern: Adapter — the rows of the state as `list-view` rows or `file-tile`s, from a shared copy, so
//! the list never reaches back into the locked state while it scrolls. Everything here is drawn in a
//! live area, so it holds only plain elements. Failures stand outside the scroller.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::icon::glyph_of;
use fs_components_widgets::transfer_list::format_bytes;
use fs_components_widgets::{
    Column, ColumnWidth, FileTile, ListPosition, ListView, Row, TileGrid, VecRows,
};

use super::actions::{ActOn, FileActions, plain_button};
use super::frame::text_height;
use crate::file_manager::context::FileView;
use crate::file_manager::labels::FileLabels;
use crate::file_manager::model::{EntryKind, FileEntry, SortKey};
use crate::file_manager::state::FilesState;

const ROW_HEIGHT: f32 = 36.0;
const MORE_HEIGHT: f32 = 48.0;
const TILE: (f32, f32) = (168.0, 152.0);
/// Below this width the list has only the name and the size.
const ALL_COLUMNS_FROM: f32 = 420.0;

fn empty_text(state: &FilesState) -> String {
    let labels = state.labels();
    if !state.term().is_empty() {
        FileLabels::fill(&labels.nothing_found, state.term())
    } else if state.in_trash() {
        labels.trash_empty.clone()
    } else {
        labels.nothing.clone()
    }
}

fn marks(state: &FilesState, entry: &FileEntry) -> Vec<String> {
    if entry.is_marked() {
        vec![state.labels().marked.clone()]
    } else {
        Vec::new()
    }
}

fn size_cell(entry: &FileEntry) -> String {
    if entry.kind == EntryKind::Folder {
        String::new()
    } else {
        format_bytes(entry.size)
    }
}

fn head_column(state: &FilesState, title: &str, width: ColumnWidth, key: SortKey) -> Column {
    let name = match key {
        SortKey::Modified => "modified",
        SortKey::Size => "size",
        SortKey::Type => "type",
        _ => "name",
    };
    let column = Column::new(title, width).sortable(name);
    if state.sort().key == key {
        column.ordered(state.sort().descending)
    } else {
        column
    }
}

fn list_view(state: &FilesState, width: f32) -> (Vec<Column>, Vec<Row>) {
    let labels = state.labels();
    let dates = state.capabilities().dates && width >= ALL_COLUMNS_FROM;
    let all = width >= ALL_COLUMNS_FROM;
    let sortable = state.capabilities().sort;
    let head = |title: &str, width, key| {
        if sortable {
            head_column(state, title, width, key)
        } else {
            Column::new(title, width)
        }
    };
    // Name, Size, Modified, Type: what a person looks for first comes first.
    let mut columns = vec![head(
        &labels.col_name,
        ColumnWidth::Share(3.0),
        SortKey::Name,
    )];
    columns.push(head(
        &labels.col_size,
        ColumnWidth::Pixels(90.0),
        SortKey::Size,
    ));
    if dates {
        columns.push(head(
            &labels.col_modified,
            ColumnWidth::Pixels(150.0),
            SortKey::Modified,
        ));
    }
    if all {
        columns.push(head(
            &labels.col_type,
            ColumnWidth::Share(1.5),
            SortKey::Type,
        ));
    }
    let rows = state
        .rows()
        .iter()
        .map(|entry| {
            let mut cells = vec![entry.name.clone(), size_cell(entry)];
            if dates {
                cells.push(entry.modified.clone().unwrap_or_default());
            }
            if all {
                cells.push(state.type_word(entry));
            }
            let row = Row::new(entry.key.clone(), cells)
                .with_glyph(glyph_of(&entry.media_type, entry.kind == EntryKind::Folder));
            if entry.is_marked() {
                row.with_mark(state.labels().marked.clone())
            } else {
                row
            }
        })
        .collect();
    (columns, rows)
}

fn grid(
    state: &FilesState,
    name: &str,
    position: &ListPosition,
    select: ActOn,
    (width, height, new_list): (f32, f32, bool),
) -> Div {
    let tiles: Vec<FileTile> = state
        .rows()
        .iter()
        .map(|e| {
            FileTile::new(e.key.clone(), e.name.clone(), state.type_word(e))
                .with_glyph(glyph_of(&e.media_type, e.kind == EntryKind::Folder))
                .with_lines(vec![size_cell(e)])
                .with_marks(marks(state, e))
        })
        .collect();
    div().w(width).h(height).child(
        TileGrid::new(Arc::new(tiles), TILE, position, name.to_string())
            .selected(state.selected_key())
            .on_select(select)
            .at_top(new_list)
            .when_empty(empty_text(state))
            .build(width, height),
    )
}

/// The work area, filling `width` x `height`. `new_list`: another list took this one's place.
pub(in crate::file_manager) fn work(
    state: &FilesState,
    name: &str,
    position: &ListPosition,
    actions: &FileActions,
    (width, height, new_list): (f32, f32, bool),
) -> Div {
    let mut area = div().flex_col().w(width).h(height).min_h(0.0);
    let mut left = height;
    if let Some(problem) = state.list_problem() {
        let tall = text_height(problem.chars().count(), width) + MORE_HEIGHT;
        left -= tall;
        area = area.child(
            div()
                .id(format!("{name}-list-problem"))
                .class("fs-files-failed-line")
                .flex_col()
                .flex_shrink_0()
                .w(width)
                .h(tall)
                .overflow_clip()
                .child(text(problem))
                .child(plain_button(
                    format!("{name}-retry"),
                    &state.labels().retry,
                    &actions.plain(FilesState::retry),
                )),
        );
    }
    if !state.unreadable().is_empty() {
        let words = FileLabels::fill(&state.labels().unreadable, &state.unreadable().join(", "));
        let tall = (text_height(words.chars().count(), width)).min(height / 3.0);
        left -= tall;
        area = area.child(
            div()
                .id(format!("{name}-unreadable"))
                .class("fs-files-failed-line")
                .flex_shrink_0()
                .w(width)
                .h(tall)
                .overflow_clip()
                .child(text(words)),
        );
    }
    let more = state.has_more();
    if more {
        left -= MORE_HEIGHT;
    }
    let left = left.max(0.0);
    let select = actions.on_key(FilesState::select);
    let shown = if state.view() == FileView::Grid {
        grid(state, name, position, select, (width, left, new_list))
    } else {
        if new_list {
            position.scroll_to_row(0);
        }
        let (columns, rows) = list_view(state, width);
        div().w(width).h(left).child(
            ListView::new(columns, Arc::new(VecRows::new(rows)))
                .row_height(ROW_HEIGHT)
                .named(name)
                .position(position)
                .selected(state.selected_key())
                .when_empty(empty_text(state))
                .on_select(select)
                .on_head(actions.on_key(FilesState::sort_by_head))
                .keys(true)
                .build(width, left),
        )
    };
    area = area.child(shown);
    if more {
        area = area.child(
            div()
                .class("fs-files-more")
                .flex_row()
                .flex_shrink_0()
                .items_center()
                .w(width)
                .h(MORE_HEIGHT)
                .child(plain_button(
                    format!("{name}-more"),
                    &state.labels().more,
                    &actions.plain(FilesState::load_more),
                )),
        );
    }
    area
}
