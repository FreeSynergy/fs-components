//! Fetching and keeping tiles: places, time limits, what is no longer wanted, what failed.
//!
//! Pattern: Object pool with a lease (a place is held while a tile is fetched) + State per tile ([`Slot`]). The draw
//! never waits: a tile that is not here is fetched on a worker thread and the view draws again when it arrives.
//! * At most [`MOST_LOADING`] tiles are fetched at once, and at most [`MOST_IN_FLIGHT`] workers exist (a source
//!   that never answers cannot be stopped, but it cannot fill the machine with threads either).
//! * A fetch that takes longer than [`Limits::load_timeout`] gives up its place: the tile is `Failed` and tried again
//!   after [`Limits::retry_after`]; the late answer of that worker is thrown away.
//! * A tile that is no longer needed (the view moved) gives up its place at the next draw.
//! * `Ok(None)` (no such tile) is kept for good; an error, a panic or bytes that do not decode are kept as `Failed`
//!   and asked for again after [`Limits::retry_after`], so a bad moment of the network is no "there is none" for ever.
//! * At most [`Limits::most_kept`] tiles are kept; the oldest go.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use fs_components_widgets::live::Live;

use crate::model::TileGeometry;
use crate::mvt::decode_tile;
use crate::surface::TileBytes;
use crate::tiles::TileId;

/// Most tiles fetched at the same time.
pub const MOST_LOADING: usize = 4;
/// Most workers alive at the same time, the ones that gave up their place and have not come back included.
pub const MOST_IN_FLIGHT: usize = 8;
/// Most decoded tiles kept; when more arrive the oldest go.
pub const MOST_KEPT: usize = 256;

/// The time and the size limits of the loader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// A fetch that takes longer gives up its place.
    pub load_timeout: Duration,
    /// A tile that failed is asked for again after this.
    pub retry_after: Duration,
    /// Most tiles kept (decoded or failed).
    pub most_kept: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            load_timeout: Duration::from_secs(20),
            retry_after: Duration::from_secs(30),
            most_kept: MOST_KEPT,
        }
    }
}

enum Slot {
    Loading { token: u64, since: Instant },
    Ready(Option<Arc<TileGeometry>>, u64),
    Failed(Instant, u64),
}

#[derive(Default)]
struct Kept {
    slots: HashMap<TileId, Slot>,
    /// Finished tiles, oldest first, with the number they were finished under.
    order: VecDeque<(TileId, u64)>,
    in_flight: usize,
    counter: u64,
}

impl Kept {
    fn loading(&self) -> usize {
        self.slots
            .values()
            .filter(|s| matches!(s, Slot::Loading { .. }))
            .count()
    }

    /// A tile is finished: it takes the place of what is in the slot and the oldest go if there are too many.
    fn finish(&mut self, id: TileId, token: u64, done: impl FnOnce(u64) -> Slot, most: usize) {
        if !matches!(self.slots.get(&id), Some(Slot::Loading { token: t, .. }) if *t == token) {
            return; // given up (timed out or no longer wanted): the late answer is thrown away
        }
        self.counter += 1;
        let number = self.counter;
        self.slots.insert(id, done(number));
        self.order.push_back((id, number));
        while self.order.len() > most {
            let Some((old, number)) = self.order.pop_front() else {
                break;
            };
            let same = match self.slots.get(&old) {
                Some(Slot::Ready(_, n) | Slot::Failed(_, n)) => *n == number,
                _ => false,
            };
            if same {
                self.slots.remove(&old);
            }
        }
    }
}

/// The tiles of one map.
pub(crate) struct Loader {
    kept: Arc<Mutex<Kept>>,
    limits: Limits,
    live: Live,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Loader {
    pub(crate) fn new(limits: Limits, live: Live) -> Self {
        Self {
            kept: Arc::default(),
            limits,
            live,
        }
    }

    /// How many tiles are decoded and kept.
    pub(crate) fn ready(&self) -> usize {
        lock(&self.kept)
            .slots
            .values()
            .filter(|s| matches!(s, Slot::Ready(..)))
            .count()
    }

    /// The tiles that the view needs now: the others give up their place, a fetch that took too long fails, a failure
    /// that is old enough is forgotten so that it is asked for again.
    pub(crate) fn retarget(&self, wanted: &[TileId]) {
        let wanted: HashSet<TileId> = wanted.iter().copied().collect();
        let now = Instant::now();
        let mut kept = lock(&self.kept);
        let limits = self.limits;
        let mut failed = Vec::new();
        kept.slots.retain(|id, slot| match slot {
            Slot::Loading { since, .. } => {
                if !wanted.contains(id) {
                    false
                } else if now.duration_since(*since) >= limits.load_timeout {
                    failed.push(*id);
                    false
                } else {
                    true
                }
            }
            Slot::Failed(at, _) => now.duration_since(*at) < limits.retry_after,
            Slot::Ready(..) => true,
        });
        for id in failed {
            kept.counter += 1;
            let number = kept.counter;
            kept.slots.insert(id, Slot::Failed(now, number));
            kept.order.push_back((id, number));
        }
    }

    /// The tile if it is here; if not, start fetching it (when there is a place) and say nothing yet.
    pub(crate) fn get(&self, source: &Arc<dyn TileBytes>, id: TileId) -> Option<Arc<TileGeometry>> {
        let mut kept = lock(&self.kept);
        match kept.slots.get(&id) {
            Some(Slot::Ready(t, _)) => return t.clone(),
            Some(Slot::Loading { .. } | Slot::Failed(..)) => return None,
            None => {}
        }
        if kept.loading() >= MOST_LOADING || kept.in_flight >= MOST_IN_FLIGHT {
            return None;
        }
        kept.counter += 1;
        let token = kept.counter;
        kept.slots.insert(
            id,
            Slot::Loading {
                token,
                since: Instant::now(),
            },
        );
        kept.in_flight += 1;
        drop(kept);
        let (source, store, live, most) = (
            Arc::clone(source),
            Arc::clone(&self.kept),
            self.live,
            self.limits.most_kept,
        );
        let worker = std::thread::Builder::new()
            .name("fs-map-tile".into())
            .spawn(move || {
                // A source that panics leaves the tile failed, never `Loading` for good.
                let answer =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        match source.tile(id) {
                            Ok(Some(bytes)) => decode_tile(&bytes).ok().map(|g| Some(Arc::new(g))),
                            Ok(None) => Some(None),
                            Err(_) => None,
                        }
                    }))
                    .unwrap_or(None);
                let mut kept = lock(&store);
                kept.in_flight = kept.in_flight.saturating_sub(1);
                kept.finish(
                    id,
                    token,
                    |n| match answer {
                        Some(tile) => Slot::Ready(tile, n),
                        None => Slot::Failed(Instant::now(), n),
                    },
                    most,
                );
                drop(kept);
                live.refresh();
            });
        if worker.is_err() {
            // No thread to be had: the tile is empty for now, and tried again later; never a panic.
            let mut kept = lock(&self.kept);
            kept.in_flight = kept.in_flight.saturating_sub(1);
            kept.finish(
                id,
                token,
                |n| Slot::Failed(Instant::now(), n),
                self.limits.most_kept,
            );
        }
        None
    }
}
