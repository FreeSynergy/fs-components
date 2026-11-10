//! The grid as an element.
//!
//! Pattern: Adapter — the sections become the tiles of the `file-tile` widget (`TileGrid`, of which only the tiles in
//! view exist); the group label is the first line below the name. The look is the widget's and the style's.

use std::sync::Arc;

use fs_components_widgets::file_tile::{FileTile, TileGrid};
use fs_components_widgets::list_view::OnSelect;
use fs_components_widgets::{Element, ListPosition};

use crate::{Grouping, Picture, sections};

/// The grid of pictures.
#[must_use]
pub struct GalleryGrid {
    tiles: Vec<FileTile>,
    selected: Option<String>,
    on_select: Option<OnSelect>,
    position: ListPosition,
    empty: String,
}

impl GalleryGrid {
    /// A grid of `pictures` grouped by `by`; `empty` is shown when there are none.
    pub fn new(pictures: Vec<Picture>, by: Grouping, empty: &str) -> Self {
        let tiles = sections(pictures, by)
            .into_iter()
            .flat_map(|s| {
                let label = s.label;
                s.pictures.into_iter().map(move |p| {
                    let lines = if label.is_empty() {
                        Vec::new()
                    } else {
                        vec![label.clone()]
                    };
                    FileTile::new(p.key.clone(), p.name.clone(), p.badge())
                        .with_lines(lines)
                        .with_marks(p.marks)
                })
            })
            .collect();
        Self {
            tiles,
            selected: None,
            on_select: None,
            position: ListPosition::new(),
            empty: empty.to_owned(),
        }
    }

    /// The tile with this key stands out.
    pub fn selected(mut self, key: Option<&str>) -> Self {
        self.selected = key.map(str::to_owned);
        self
    }

    /// Report the key of the tile the person chose.
    pub fn on_select(mut self, handler: OnSelect) -> Self {
        self.on_select = Some(handler);
        self
    }

    /// The scroll position to keep between builds.
    pub fn position(mut self, position: &ListPosition) -> Self {
        self.position = position.clone();
        self
    }

    /// The keys in the order shown.
    pub fn keys(&self) -> Vec<&str> {
        self.tiles.iter().map(|t| t.key.as_str()).collect()
    }

    /// The element, `width` x `height`.
    pub fn build(self, width: f32, height: f32) -> impl Element + use<> {
        let mut grid = TileGrid::new(
            Arc::new(self.tiles),
            (160.0, 120.0),
            &self.position,
            "fs-gallery-grid",
        )
        .selected(self.selected.as_deref())
        .when_empty(self.empty);
        if let Some(handler) = self.on_select {
            grid = grid.on_select(handler);
        }
        grid.build(width, height)
    }
}
