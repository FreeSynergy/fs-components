//! The picture of the map for one build: which tiles, which markers, and what went wrong.
//!
//! Pattern: Builder — reads the tiles the view needs through the cache, makes the markers from the
//! places loaded, and collects what must be said: a tile that could not be read is named, shapes left out
//! are counted. A missing tile is not an error and is not said. One bad tile costs itself only.

use std::sync::Arc;

use fs_components_widgets::map_view::{
    MapView, Marker, NeededTile, TileGeometry, TileId, TileSet, needed_tiles,
};

use super::labels::MapLabels;
use super::model::TileProblem;
use super::state::MapState;
use super::tile_store::Reading;

/// More places than this are labelled only when chosen.
const LABELLED: usize = 12;
/// The most lines of trouble shown over the map.
const MOST_LINES: usize = 3;

/// What went wrong with one picture of the map.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Picture {
    /// Tiles that could not be read, with why.
    pub problems: Vec<TileProblem>,
    /// Shapes in view that were left out because there are too many to draw.
    pub omitted: usize,
    /// More tiles were needed than are drawn at once.
    pub too_wide: bool,
}

impl Picture {
    /// What to tell the person, one line each, at most three.
    pub fn lines(&self, labels: &MapLabels) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(first) = self.problems.first() {
            let id = first.id;
            let named = format!("{}/{}/{}: {}", id.z, id.x, id.y, first.why);
            let mut line = MapLabels::fill(&labels.tile_failed, &named);
            if self.problems.len() > 1 {
                let more = (self.problems.len() - 1).to_string();
                line.push(' ');
                line.push_str(&MapLabels::fill(&labels.tiles_more, &more));
            }
            lines.push(line);
        }
        if self.too_wide {
            lines.push(labels.too_wide.clone());
        }
        if self.omitted > 0 {
            lines.push(MapLabels::fill(&labels.too_many, &self.omitted.to_string()));
        }
        lines.truncate(MOST_LINES);
        lines
    }
}

fn tiles_of(state: &MapState, set: &TileSet, picture: &mut Picture) -> Vec<(NeededTile, Arc<TileGeometry>)> {
    let mut tiles = Vec::new();
    let mut seen: Vec<TileId> = Vec::new();
    for needed in &set.tiles {
        match state.tiles.get(needed.id, state.source.as_ref()) {
            Reading::Tile(tile) => tiles.push((*needed, tile)),
            Reading::Missing => {}
            Reading::Failed(why) => {
                // The same tile may be drawn twice where the world repeats; it is named once.
                if !seen.contains(&needed.id) {
                    seen.push(needed.id);
                    picture.problems.push(TileProblem { id: needed.id, why });
                }
            }
        }
    }
    tiles
}

fn markers_of(state: &MapState) -> Vec<Marker> {
    let few = state.rows.len() <= LABELLED;
    state
        .rows
        .iter()
        .filter_map(|row| {
            let (lon, lat) = row.at?;
            let selected = state.selected() == Some(row.key.as_str());
            let mut marker = Marker::new(row.key.clone(), lon, lat);
            if !row.marks.is_empty() {
                marker = marker.marked();
            } else if few || selected {
                marker = marker.labelled(row.name.clone());
            }
            Some(if selected { marker.selected() } else { marker })
        })
        .collect()
}

/// The map widget for `state` at `width` x `height`, and what is to be said about it. Reads tiles.
pub(super) fn view(state: &MapState, name: &str, width: f32, height: f32) -> (MapView, Picture) {
    let mut picture = Picture::default();
    let set = match state.range {
        Some(range) => needed_tiles(
            state.position,
            f64::from(width),
            f64::from(height),
            range.min_zoom,
            range.max_zoom,
        ),
        None => TileSet {
            tiles: Vec::new(),
            truncated: false,
        },
    };
    picture.too_wide = set.truncated;
    let tiles = tiles_of(state, &set, &mut picture);
    let view = MapView::new(name, state.position, state.labels.view.clone())
        .tiles(tiles)
        .markers(markers_of(state));
    picture.omitted = view.scene(width, height).omitted;
    (view, picture)
}
