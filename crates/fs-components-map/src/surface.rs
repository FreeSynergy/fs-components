//! The map as a screen of the window shell: position, markers and tiles kept outside the element.
//!
//! Pattern: Strategy consumer + Observer — the caller gives two traits ([`MapData`]: the markers and what happens when one
//! is chosen; [`TileBytes`]: raw vector tiles); tiles are fetched on worker threads (at most [`MOST_LOADING`] at once,
//! the draw never waits), decoded once and kept (at most [`MOST_KEPT`]); when one arrives the view draws again
//! (`Live`, the observer of the widgets). Pan and zoom change the kept position and draw again.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use fs_components_app::{Element, Surface};
use fs_components_widgets::live::Live;

use crate::model::{Marker, TileGeometry};
use crate::mvt::decode_tile;
use crate::position::MapPosition;
use crate::tiles::{TileId, needed_tiles};
use crate::widget::{MapView, MapViewLabels};

/// Most tiles fetched at the same time.
pub const MOST_LOADING: usize = 4;
/// Most decoded tiles kept; when more arrive the oldest go.
pub const MOST_KEPT: usize = 256;

/// What the map shows besides tiles.
pub trait MapData: Send + Sync {
    /// The markers now; asked at every draw, so it must be quick.
    fn markers(&self) -> Vec<Marker>;
    /// A marker was chosen (by its key).
    fn picked(&self, key: &str);
}

/// Where vector tiles come from. Called on a worker thread, so it may take its time.
pub trait TileBytes: Send + Sync {
    /// The uncompressed MVT of a tile; `Ok(None)` = no such tile (not an error).
    ///
    /// # Errors
    /// The reason, for the log; the tile is then left empty and not asked for again.
    fn tile(&self, id: TileId) -> Result<Option<Vec<u8>>, String>;
    /// The zoom levels the source has.
    fn zoom_range(&self) -> (u8, u8) {
        (0, 14)
    }
}

enum Slot {
    Loading,
    Ready(Option<Arc<TileGeometry>>),
}

#[derive(Default)]
struct Kept {
    slots: HashMap<TileId, Slot>,
    order: Vec<TileId>,
}

/// The map as a [`Surface`]; cheap to clone, every clone is the same map.
#[derive(Clone)]
pub struct MapSurface(Arc<Inner>);

struct Inner {
    name: String,
    labels: MapViewLabels,
    data: Arc<dyn MapData>,
    tiles: Option<Arc<dyn TileBytes>>,
    position: Mutex<MapPosition>,
    kept: Arc<Mutex<Kept>>,
    live: Live,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl MapSurface {
    /// A map named `name` (element ids begin with it), starting at `start`.
    pub fn new(
        name: impl Into<String>,
        labels: MapViewLabels,
        data: Arc<dyn MapData>,
        tiles: Option<Arc<dyn TileBytes>>,
        start: MapPosition,
    ) -> Self {
        Self(Arc::new(Inner {
            name: name.into(),
            labels,
            data,
            tiles,
            position: Mutex::new(start),
            kept: Arc::default(),
            live: Live::new(),
        }))
    }

    /// Where the map looks now.
    #[must_use]
    pub fn position(&self) -> MapPosition {
        *lock(&self.0.position)
    }

    /// Look somewhere else and draw again.
    pub fn go_to(&self, position: MapPosition) {
        *lock(&self.0.position) = position;
        self.0.live.refresh_now();
    }

    /// Draw again (the markers changed).
    pub fn refresh(&self) {
        self.0.live.refresh_now();
    }

    /// How many tiles are decoded and kept.
    #[must_use]
    pub fn tiles_kept(&self) -> usize {
        lock(&self.0.kept)
            .slots
            .values()
            .filter(|s| matches!(s, Slot::Ready(_)))
            .count()
    }

    fn tile(&self, source: &Arc<dyn TileBytes>, id: TileId) -> Option<Arc<TileGeometry>> {
        let mut kept = lock(&self.0.kept);
        match kept.slots.get(&id) {
            Some(Slot::Ready(t)) => return t.clone(),
            Some(Slot::Loading) => return None,
            None => {}
        }
        let loading = kept
            .slots
            .values()
            .filter(|s| matches!(s, Slot::Loading))
            .count();
        if loading >= MOST_LOADING {
            return None;
        }
        kept.slots.insert(id, Slot::Loading);
        drop(kept);
        let (source, store, live) = (Arc::clone(source), Arc::clone(&self.0.kept), self.0.live);
        std::thread::spawn(move || {
            // A tile source that panics leaves the tile empty, never `Loading` for good (it would use up a place of
            // the loader).
            let geometry =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match source.tile(id) {
                    Ok(Some(bytes)) => decode_tile(&bytes).ok().map(Arc::new),
                    Ok(None) | Err(_) => None,
                }))
                .unwrap_or(None);
            let mut kept = lock(&store);
            kept.slots.insert(id, Slot::Ready(geometry));
            kept.order.push(id);
            while kept.order.len() > MOST_KEPT {
                let old = kept.order.remove(0);
                kept.slots.remove(&old);
            }
            drop(kept);
            live.refresh();
        });
        None
    }

    fn view(&self, width: f32, height: f32) -> impl Element + use<> {
        let position = self.position();
        let mut shown = Vec::new();
        if let Some(source) = &self.0.tiles {
            let (low, high) = source.zoom_range();
            for needed in
                needed_tiles(position, f64::from(width), f64::from(height), low, high).tiles
            {
                if let Some(t) = self.tile(source, needed.id) {
                    shown.push((needed, t));
                }
            }
        }
        let (on_move, on_pick) = (self.clone(), Arc::clone(&self.0.data));
        MapView::new(self.0.name.clone(), position, self.0.labels.clone())
            .tiles(shown)
            .markers(self.0.data.markers())
            .on_change(Arc::new(move |p| on_move.go_to(p)))
            .on_pick(Arc::new(move |k| on_pick.picked(k)))
            .build(width, height)
    }
}

impl Surface for MapSurface {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        let me = self.clone();
        Box::new(self.0.live.region("map", move || me.view(width, height)))
    }

    fn styles(&self) -> String {
        crate::STYLES.to_owned()
    }
}
