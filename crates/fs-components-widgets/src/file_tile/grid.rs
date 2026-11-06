//! The grid of tiles.
//!
//! Pattern: Adapter over Blinc's scroll container and keyed rows, as `list-view` does it, and on its
//! `ListPosition`: a box as tall as all rows, in it only the rows of tiles in view. Grid rows are of
//! one height, so a very long grid costs what a short one costs (the tiles are in memory).

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::Scroll;

use super::tile::FileTile;
use crate::Element;
use crate::keys::{Key, KeyEvent, bind_keys};
use crate::list_view::{ListPosition, OnSelect};

/// The tile a key leads to, from `cursor` (none chosen yet: the first), among `count` tiles in rows of
/// `columns`, with `page` rows in view. `None` if there are none or the grid does not use this key.
pub fn grid_target(
    key: Key,
    cursor: Option<usize>,
    count: usize,
    columns: usize,
    page: usize,
) -> Option<usize> {
    let last = count.checked_sub(1)?;
    let columns = columns.max(1);
    let Some(at) = cursor.map(|c| c.min(last)) else {
        return matches!(
            key,
            Key::Up | Key::Down | Key::Left | Key::Right | Key::Home | Key::PageUp | Key::PageDown
        )
        .then_some(0)
        .or((key == Key::End).then_some(last));
    };
    match key {
        Key::Home => Some(0),
        Key::End => Some(last),
        Key::Left => Some(at.saturating_sub(1)),
        Key::Right => Some((at + 1).min(last)),
        Key::Up => Some(at.checked_sub(columns).unwrap_or(at)),
        Key::Down => Some(if at + columns <= last {
            at + columns
        } else {
            at
        }),
        Key::PageUp => Some(at.saturating_sub(columns * page.max(1))),
        Key::PageDown => Some((at + columns * page.max(1)).min(last)),
        _ => None,
    }
}

/// A grid of [`FileTile`]s, ready to be built into a layout.
#[must_use]
pub struct TileGrid {
    tiles: Arc<Vec<FileTile>>,
    size: (f32, f32),
    gap: f32,
    selected: Option<String>,
    on_select: Option<OnSelect>,
    position: ListPosition,
    at_top: bool,
    name: String,
    empty: String,
}

impl TileGrid {
    /// A grid of `tiles`, each `tile_width` x `tile_height`. Element ids: `<name>`, `<name>-<key>`.
    pub fn new(
        tiles: Arc<Vec<FileTile>>,
        (tile_width, tile_height): (f32, f32),
        position: &ListPosition,
        name: impl Into<String>,
    ) -> Self {
        Self {
            tiles,
            size: (tile_width, tile_height),
            gap: 8.0,
            selected: None,
            on_select: None,
            position: position.clone(),
            at_top: false,
            name: name.into(),
            empty: String::new(),
        }
    }

    /// The tile with this key stands out. The caller answers `on_select` by building again.
    pub fn selected(mut self, key: Option<&str>) -> Self {
        self.selected = key.map(str::to_owned);
        self
    }

    /// Report the key of the tile the person chose, by a click or a key.
    pub fn on_select(mut self, handler: OnSelect) -> Self {
        self.on_select = Some(handler);
        self
    }

    /// Start at the top: other tiles took the place of the ones before.
    pub fn at_top(mut self, top: bool) -> Self {
        self.at_top = top;
        self
    }

    /// The text to show when there are no tiles; it comes from the caller.
    pub fn when_empty(mut self, text: impl Into<String>) -> Self {
        self.empty = text.into();
        self
    }

    /// How many tiles fit next to each other in `width`.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    pub fn columns(&self, width: f32) -> usize {
        ((width + self.gap) / (self.size.0 + self.gap))
            .floor()
            .max(1.0) as usize
    }

    /// The element to place in a layout, filling `width` x `height`.
    #[allow(clippy::cast_precision_loss)]
    pub fn build(self, width: f32, height: f32) -> impl Element + use<> {
        let root = div()
            .id(self.name.clone())
            .class("fs-tiles")
            .flex_col()
            .w(width)
            .h(height)
            .min_h(0.0);
        if self.tiles.is_empty() {
            return root.child(
                div()
                    .class("fs-tiles-empty")
                    .child(text(self.empty.clone())),
            );
        }
        let columns = self.columns(width);
        let count = self.tiles.len();
        let rows = count.div_ceil(columns);
        let row_height = self.size.1 + self.gap;
        let total = rows as f32 * row_height;
        let was = if self.at_top {
            0.0
        } else {
            self.position.offset()
        };
        let offset = was.clamp(0.0, (total - height).max(0.0));
        self.position.apply_offset(offset);
        let body = self.body(columns, rows, offset, height);
        let scroll = Scroll::with_physics(Arc::clone(&self.position.physics))
            .bind(&self.position.scroll)
            .w(width)
            .h(height)
            .content(body.w(width));
        let root = root.child(scroll);
        self.with_keys(root, columns, (height / row_height).max(1.0))
    }

    #[allow(clippy::cast_precision_loss)]
    fn body(&self, columns: usize, rows: usize, offset: f32, view: f32) -> Div {
        let row_height = self.size.1 + self.gap;
        let window = self.position.window(offset, (view, row_height, rows));
        let (tiles, size, gap) = (Arc::clone(&self.tiles), self.size, self.gap);
        let (selected, on_select, name) = (
            self.selected.clone(),
            self.on_select.clone(),
            self.name.clone(),
        );
        div()
            .relative()
            .h(rows as f32 * row_height)
            .flex_shrink_0()
            .for_each(
                &window,
                |index: &usize| *index,
                move |row: usize| {
                    let mut line = div().flex_row().gap_px(gap);
                    for tile in tiles.iter().skip(row * columns).take(columns) {
                        let chosen = selected.as_deref() == Some(tile.key.as_str());
                        let mut holder = tile.build(size.0, size.1, chosen);
                        holder = holder.id(format!("{name}-{}", tile.key));
                        if let Some(handler) = &on_select {
                            let (handler, key) = (Arc::clone(handler), tile.key.clone());
                            holder = holder.on_click(move |_| handler(&key));
                        }
                        line = line.child(holder);
                    }
                    line.absolute().left(0.0).top(row as f32 * row_height)
                },
            )
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    fn with_keys(&self, root: Div, columns: usize, page: f32) -> Div {
        let (tiles, selected, on_select, position) = (
            Arc::clone(&self.tiles),
            self.selected.clone(),
            self.on_select.clone(),
            self.position.clone(),
        );
        let row_height = self.size.1 + self.gap;
        let view = page * row_height;
        let handler = Arc::new(move |event: KeyEvent| {
            let cursor = selected
                .as_ref()
                .and_then(|key| tiles.iter().position(|t| &t.key == key));
            let Some(target) = grid_target(event.key, cursor, tiles.len(), columns, page as usize)
            else {
                return false;
            };
            let row = target / columns;
            let (top, bottom) = (row as f32 * row_height, (row + 1) as f32 * row_height);
            let offset = position.offset();
            if top < offset {
                position.apply_offset(top);
            } else if bottom > offset + view {
                position.apply_offset(bottom - view);
            }
            if let Some(on_select) = &on_select {
                on_select(&tiles[target].key);
            }
            true
        });
        bind_keys(root, handler)
    }
}
