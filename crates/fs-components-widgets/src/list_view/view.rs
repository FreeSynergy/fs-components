//! The list as elements.
//!
//! Pattern: Adapter over Blinc's scroll container and keyed rows. A box as tall as all rows sits in
//! the scroll container; only the rows in view exist inside it, placed by their index. When the
//! scroll position changes, the rows that left are dropped and the ones that came are made. This is
//! how `blinc_cn`'s long select works; see DESIGN.md for why `virtual_list.rs` is not used.

use std::sync::Arc;

use blinc_core::reactive::{ReactiveGraph, computed};
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::has_focused_text_input;
use blinc_layout::widgets::scroll::Scroll;

use super::cache::RowCache;
use super::keys::{ListKey, key_target};
use super::model::{ListView, OnSelect, Row};
use super::position::ListPosition;
use super::window::{column_widths, visible_rows};
use crate::Element;

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
        let count = self.source.count();
        let widths = Arc::new(column_widths(&self.columns, width));
        let with_head = self.columns.iter().any(|c| !c.title.is_empty());
        let head = with_head.then(|| head(&self, &widths));
        let view = height - if with_head { HEAD_HEIGHT } else { 0.0 };
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
    let watch = position.scroll.watch_y(offset);
    if watch.try_get().is_none_or(|y| (y - offset).abs() > 0.5) {
        watch.set(offset);
    }
    let window = computed(move |graph: &ReactiveGraph| {
        let y = graph.get(watch).unwrap_or(offset);
        visible_rows(y, view, row_height, count).collect::<Vec<usize>>()
    });
    let cache = RowCache::new(Arc::clone(&list.source));
    let shared = Shared {
        widths: Arc::clone(widths),
        selected: list.selected.clone(),
        on_select: list.on_select.clone(),
        position: position.clone(),
        name: list.name.clone(),
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
            element = element.child(
                div()
                    .class("fs-list-cell")
                    .w(*width)
                    .child(text(cell).no_wrap().no_cursor()),
            );
        }
        let (key, on_select, position) = (
            row.key.clone(),
            self.on_select.clone(),
            self.position.clone(),
        );
        element.on_click(move |_| {
            position.set_cursor(Some(index));
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

/// Keys move the row under the keyboard, keep it in view and report it as chosen.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn with_keys(root: Div, list: &ListView, position: ListPosition, count: usize, view: f32) -> Div {
    let row_height = list.row_height;
    let page = (view / row_height) as usize;
    let (source, on_select) = (Arc::clone(&list.source), list.on_select.clone());
    root.on_key_down(move |event| {
        let Some(key) = ListKey::from_code(event.key_code) else {
            return;
        };
        if has_focused_text_input() {
            return;
        }
        let Some(row) = key_target(key, position.cursor(), count, page) else {
            return;
        };
        position.set_cursor(Some(row));
        let (top, bottom) = (row as f32 * row_height, (row + 1) as f32 * row_height);
        let offset = position.offset();
        if top < offset {
            position.apply_offset(top);
        } else if bottom > offset + view {
            position.apply_offset(bottom - view);
        }
        if let (Some(on_select), Ok(rows)) = (&on_select, source.rows(row, 1))
            && let Some(found) = rows.first()
        {
            on_select(&found.key);
        }
    })
}
