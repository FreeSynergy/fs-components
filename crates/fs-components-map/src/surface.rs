//! The map as a screen of the window shell: position, markers and tiles kept outside the element.
//!
//! Pattern: Strategy consumer + Observer — the caller gives two traits ([`MapData`]: the markers and what happens when one
//! is chosen; [`TileBytes`]: raw vector tiles); tiles are fetched on worker threads (a few at once, with time limits: see `loader`;
//! the draw never waits), decoded once and kept; when one arrives the view draws again
//! (`Live`, the observer of the widgets). Pan and zoom change the kept position and draw again.

use std::sync::{Arc, Mutex, PoisonError};

use fs_components_app::{Element, Surface};
use fs_components_widgets::live::Live;

use crate::loader::{Limits, Loader};
use crate::model::Marker;
use crate::position::MapPosition;
use crate::tiles::{TileId, needed_tiles};
use crate::widget::{MapView, MapViewLabels};

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
    /// Who made the data, as the source's licence asks it to be shown on the map (for OpenStreetMap:
    /// "© OpenStreetMap contributors"). The map shows it in a corner, always, while it is `Some`; the program that
    /// takes tiles from a source gives it here, from the data it has, so no program can forget it.
    fn attribution(&self) -> Option<String> {
        None
    }
    /// The zoom levels the source has.
    fn zoom_range(&self) -> (u8, u8) {
        (0, 14)
    }
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
    loader: Loader,
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
        let live = Live::new();
        Self(Arc::new(Inner {
            name: name.into(),
            labels,
            data,
            tiles,
            position: Mutex::new(start),
            loader: Loader::new(Limits::default(), live),
            live,
        }))
    }

    /// The same map with other time and size limits for loading tiles (call it first, before the map is cloned).
    #[must_use]
    pub fn with_limits(mut self, limits: Limits) -> Self {
        if let Some(inner) = Arc::get_mut(&mut self.0) {
            inner.loader = Loader::new(limits, inner.live);
        }
        self
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
        self.0.loader.ready()
    }

    fn view(&self, width: f32, height: f32) -> impl Element + use<> {
        let position = self.position();
        let mut shown = Vec::new();
        if let Some(source) = &self.0.tiles {
            let (low, high) = source.zoom_range();
            let needed =
                needed_tiles(position, f64::from(width), f64::from(height), low, high).tiles;
            // What is no longer needed gives up its place, before the places are handed out again.
            self.0
                .loader
                .retarget(&needed.iter().map(|n| n.id).collect::<Vec<_>>());
            for needed in needed {
                if let Some(t) = self.0.loader.get(source, needed.id) {
                    shown.push((needed, t));
                }
            }
        }
        let (on_move, on_pick) = (self.clone(), Arc::clone(&self.0.data));
        let attribution = self.0.tiles.as_ref().and_then(|t| t.attribution());
        MapView::new(self.0.name.clone(), position, self.0.labels.clone())
            .tiles(shown)
            .attribution(attribution)
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
