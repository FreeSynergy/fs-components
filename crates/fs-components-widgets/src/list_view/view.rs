//! The list as elements.
//!
//! Pattern: Adapter over Blinc's scroll container and keyed rows. A box as tall as all rows sits in
//! the scroll container; only the rows in view exist inside it, placed by their index. When the
//! scroll position changes, the rows that left are dropped and the ones that came are made. This is
//! how `blinc_cn`'s long select works; see DESIGN.md for why `virtual_list.rs` is not used.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::Scroll;

use super::cache::RowCache;
use super::cell::{FIRST_CELL_MARK_ROOM, fit_cell};
use super::keys::key_target;
use super::model::{ListView, OnSelect, Row, RowSource};
use super::position::ListPosition;
use super::window::column_widths;
use crate::Element;
use crate::keys::{Key, KeyEvent, bind_keys};

/// Height of the row of headings in pixels.
const HEAD_HEIGHT: f32 = 28.0;

impl ListView {
    /// The element to place in a layout, filling `width` x `height`.
    #[allow(clippy::cast_precision_loss)] // row positions: see "Limits" in DESIGN.md
    pub fn build(self, width: f32, height: f32) -> impl Element + use<> {
        let root = div()
            .class("fs-list")
            .flex_col()
            .w(width)
            .h(height)
            .min_h(0.0);
        let root = match &self.name {
            Some(name) => root.id(name),
            None => root,
        };
        let count = self.source.count();
        let widths = Arc::new(column_widths(&self.columns, width));
        let with_head = self.columns.iter().any(|c| !c.title.is_empty());
        let head = with_head.then(|| head(&self, &widths));
        let view = (height - if with_head { HEAD_HEIGHT } else { 0.0 }).max(0.0);
        let root = match head {
            Some(head) => root.child(head),
            None => root,
        };
        if count == 0 {
            let message = self.empty.clone().unwrap_or_default();
            return root.child(div().class("fs-list-empty").child(text(message)));
        }
        let position = self.position.clone().unwrap_or_default();
        let row_height = self.row_height;
        let total = count as f32 * row_height;
        if let Some(row) = position.arrived(self.selected.as_deref()) {
            if let Some(key) = &self.selected {
                position.set_cursor_on(row, key);
            }
            bring_into_view(&position, row, row_height, view);
        }
        let offset = settle(&position, total, view, row_height);
        let rows = rows(&self, &position, &widths, width, count, offset, view);
        let body = Scroll::with_physics(Arc::clone(&position.physics))
            .bind(&position.scroll)
            .w(width)
            .h(view)
            .content(rows);
        let root = root.child(body);
        if self.keys {
            with_keys(root, &self, position, count, view)
        } else {
            root
        }
    }
}

/// Where the list stands now that it is built: the row asked for, else where it was, within the
/// rows there are now.
#[allow(clippy::cast_precision_loss)]
fn settle(position: &ListPosition, total: f32, view: f32, row_height: f32) -> f32 {
    let was = position
        .take_wanted()
        .map_or_else(|| position.offset(), |row| row as f32 * row_height);
    let offset = was.clamp(0.0, (total - view).max(0.0));
    position.apply_offset(offset);
    offset
}

/// The box as tall as all rows, holding the rows in view.
#[allow(clippy::cast_precision_loss)]
fn rows(
    list: &ListView,
    position: &ListPosition,
    widths: &Arc<Vec<f32>>,
    width: f32,
    count: usize,
    offset: f32,
    view: f32,
) -> Div {
    let row_height = list.row_height;
    let window = position.window(offset, (view, row_height, count));
    let cache = RowCache::new(Arc::clone(&list.source));
    let shared = Shared {
        widths: Arc::clone(widths),
        selected: list.selected.clone(),
        on_select: list.on_select.clone(),
        position: position.clone(),
        name: list.name.clone(),
        caller_answers: list.caller_selects && list.on_select.is_some(),
    };
    div()
        .relative()
        .w(width)
        .h(count as f32 * row_height)
        .flex_shrink_0()
        .for_each(
            &window,
            |index: &usize| *index,
            move |index: usize| {
                let element = match cache.row(index) {
                    Ok(Some(row)) => shared.row_element(&row, index),
                    Ok(None) => div(),
                    Err(why) => div().class("fs-list-failed").child(text(why)),
                };
                element
                    .absolute()
                    .left(0.0)
                    .top(index as f32 * row_height)
                    .w(width)
                    .h(row_height)
            },
        )
}

/// What every row of one list needs.
struct Shared {
    widths: Arc<Vec<f32>>,
    selected: Option<String>,
    on_select: Option<OnSelect>,
    position: ListPosition,
    name: Option<String>,
    caller_answers: bool,
}

impl Shared {
    fn row_element(&self, row: &Row, index: usize) -> Div {
        let mut element = div().class("fs-list-row").flex_row();
        if let Some(name) = &self.name {
            element = element.id(format!("{name}-{}", row.key));
        }
        if self.selected.as_deref() == Some(row.key.as_str()) {
            element = element.class("fs-list-row-selected");
        }
        for (column, width) in self.widths.iter().enumerate() {
            let cell = row.cells.get(column).map_or("", String::as_str);
            let mark = row.mark.as_ref().filter(|_| column == 0);
            let room = width
                - if mark.is_some() {
                    FIRST_CELL_MARK_ROOM
                } else {
                    0.0
                };
            let mut holder = div()
                .class("fs-list-cell")
                .flex_row()
                .items_center()
                .w(*width);
            if let Some(mark) = mark {
                holder = holder.child(
                    div()
                        .class("fs-list-mark")
                        .child(text(mark).no_wrap().no_cursor()),
                );
            }
            element = element.child(holder.child(text(fit_cell(cell, room)).no_wrap().no_cursor()));
        }
        let (key, on_select, position) = (
            row.key.clone(),
            self.on_select.clone(),
            self.position.clone(),
        );
        let caller_answers = self.caller_answers;
        element.on_click(move |_| {
            // A caller may refuse: only a build with this row selected moves the cursor then.
            if caller_answers {
                position.going_to(&key, index);
            } else {
                position.set_cursor_on(index, &key);
            }
            if let Some(on_select) = &on_select {
                on_select(&key);
            }
        })
    }
}

fn head(list: &ListView, widths: &[f32]) -> Div {
    let mut element = div().class("fs-list-head").flex_row().h(HEAD_HEIGHT);
    for (column, width) in list.columns.iter().zip(widths) {
        element = element.child(
            div()
                .class("fs-list-cell")
                .w(*width)
                .child(text(&column.title).no_wrap().no_cursor()),
        );
    }
    element
}

/// Keys report the row they lead to. A caller that selects decides: only a build with that row
/// selected moves anything, so a refusal moves nothing. Otherwise they move the list themselves.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn with_keys(root: Div, list: &ListView, position: ListPosition, count: usize, view: f32) -> Div {
    let row_height = list.row_height;
    let page = (view / row_height) as usize;
    let caller_answers = list.caller_selects && list.on_select.is_some();
    let (source, on_select, selected) = (
        Arc::clone(&list.source),
        list.on_walk.clone().or_else(|| list.on_select.clone()),
        list.selected.clone(),
    );
    let on_open = list.on_open.clone();
    let handler = Arc::new(move |event: KeyEvent| {
        if let (Some(on_open), Key::Activate | Key::Enter | Key::Space) = (&on_open, event.key) {
            if let Some(key) = &selected {
                on_open(key);
            }
            return true;
        }
        // A caller that selects has the say; without one the keys go on from where they were, and
        // a selection nobody can change is only where they begin.
        let base = if caller_answers {
            selected.clone()
        } else {
            position.cursor_key().or_else(|| selected.clone())
        };
        let cursor = if caller_answers && selected.is_none() {
            None
        } else {
            position.cursor()
        };
        let start = start_row(source.as_ref(), base, cursor);
        let Some(row) = key_target(event.key, start, count, page) else {
            return false;
        };
        let Ok(rows) = source.rows(row, 1) else {
            return true;
        };
        let Some(found) = rows.first() else {
            return true;
        };
        if caller_answers {
            position.going_to(&found.key, row);
        } else {
            position.set_cursor_on(row, &found.key);
            bring_into_view(&position, row, row_height, view);
        }
        if let Some(on_select) = &on_select {
            on_select(&found.key);
        }
        true
    });
    bind_keys(root, handler)
}

/// The row the keys start from: `base` is the key to find, the selected row if the caller selects
/// and else the last row they reached. A row that is no longer there (hidden by a search, deleted)
/// gives no start, so they begin at the first row. Without a key the cursor is taken as it is.
fn start_row(source: &dyn RowSource, base: Option<String>, cursor: Option<usize>) -> Option<usize> {
    let Some(key) = base else {
        return cursor;
    };
    let still_there = cursor.filter(|row| {
        source
            .rows(*row, 1)
            .is_ok_and(|rows| rows.first().is_some_and(|r| r.key == key))
    });
    still_there.or_else(|| source.position_of(&key))
}

#[allow(clippy::cast_precision_loss)]
fn bring_into_view(position: &ListPosition, row: usize, row_height: f32, view: f32) {
    let (top, bottom) = (row as f32 * row_height, (row + 1) as f32 * row_height);
    let offset = position.offset();
    if top < offset {
        position.apply_offset(top);
    } else if bottom > offset + view {
        position.apply_offset(bottom - view);
    }
}
