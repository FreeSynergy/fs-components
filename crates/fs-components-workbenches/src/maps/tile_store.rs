//! The tiles read so far.
//!
//! Pattern: Decorator-like cache over a Strategy (`MapSource::tile`) — a tile is read once and kept (the
//! least recently used go first, so memory stays bounded). A tile that is not there is remembered as
//! missing, one that could not be read is remembered with the reason, so a redraw does not ask the
//! source again for what it has already answered; `forget_failures` lets the person try again. The
//! lock is held only to look up or to store, never while the source is asked: a slow source must not
//! block another reader, and a source that calls back cannot deadlock it.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, PoisonError};

use fs_components_widgets::map_view::{TileGeometry, TileId};

use super::source::MapSource;

/// How many tiles are kept.
pub const KEPT: usize = 256;

/// What the source said about a tile.
#[derive(Clone, Debug)]
pub enum Reading {
    /// The tile.
    Tile(Arc<TileGeometry>),
    /// There is none, which is not an error.
    Missing,
    /// It could not be read, and why.
    Failed(String),
}

#[derive(Default)]
struct Kept {
    entries: HashMap<TileId, Reading>,
    order: VecDeque<TileId>,
}

/// The cache. Clones share it.
#[derive(Clone, Default)]
pub struct TileStore(Arc<Mutex<Kept>>);

impl TileStore {
    /// What the source says about `id`, asking it only if it has not been asked.
    pub fn get(&self, id: TileId, source: &dyn MapSource) -> Reading {
        if let Some(found) = self.lookup(id) {
            return found;
        }
        // Not under the lock: the source may take long.
        let reading = match source.tile(id) {
            Ok(Some(tile)) => Reading::Tile(tile),
            Ok(None) => Reading::Missing,
            Err(e) => Reading::Failed(e.message),
        };
        let mut kept = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if kept.entries.insert(id, reading.clone()).is_none() {
            kept.order.push_back(id);
        }
        while kept.order.len() > KEPT {
            if let Some(oldest) = kept.order.pop_front() {
                kept.entries.remove(&oldest);
            }
        }
        reading
    }

    fn lookup(&self, id: TileId) -> Option<Reading> {
        let mut kept = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let found = kept.entries.get(&id).cloned()?;
        // Used now: it goes to the back of the line.
        kept.order.retain(|other| *other != id);
        kept.order.push_back(id);
        Some(found)
    }

    /// Forget every failure and every missing tile, so the next draw asks again.
    pub fn forget_failures(&self) {
        let mut kept = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        kept.entries.retain(|_, r| matches!(r, Reading::Tile(_)));
        let entries = std::mem::take(&mut kept.entries);
        kept.order.retain(|id| entries.contains_key(id));
        kept.entries = entries;
    }

    /// How many tiles are kept.
    pub fn len(&self) -> usize {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .entries
            .len()
    }

    /// Is nothing kept?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
