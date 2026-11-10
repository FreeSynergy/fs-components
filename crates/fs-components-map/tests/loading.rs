//! Fetching tiles: a hanging source gives up its place, what is no longer wanted gives up its place, a failure is not
//! kept for good, a panic leaves the tile empty, and the oldest tiles go when there are too many.
//!
//! Pattern: none (tests). The sources are made to answer, hang, fail or panic on demand.

mod common;

use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::serial;
use fs_components_app::Surface;
use fs_components_map::{
    Limits, MOST_LOADING, MapData, MapPosition, MapSurface, MapViewLabels, Marker, TileBytes,
    TileId,
};
use mvt::{GeomEncoder, GeomType, Tile};

fn fixture() -> Vec<u8> {
    let mut tile = Tile::new(4096);
    let layer = tile.create_layer("roads");
    let road = GeomEncoder::new(GeomType::Linestring)
        .point(0.0, 2000.0)
        .and_then(|g| g.point(4000.0, 2100.0))
        .and_then(GeomEncoder::encode)
        .unwrap();
    tile.add_layer(layer.into_feature(road).into_layer())
        .unwrap();
    tile.to_bytes().unwrap()
}

struct Nothing;
impl MapData for Nothing {
    fn markers(&self) -> Vec<Marker> {
        Vec::new()
    }
    fn picked(&self, _: &str) {}
}

/// What a source does with a tile.
#[derive(Clone, Copy)]
enum Mode {
    Answer,
    Hang,
    Fail,
    Panic,
    Missing,
}

struct Scripted {
    mode: Mutex<Mode>,
    asked: Mutex<Vec<TileId>>,
    running: AtomicUsize,
    most_running: AtomicUsize,
    release: Mutex<bool>,
    bytes: Vec<u8>,
}

impl Scripted {
    fn new(mode: Mode) -> Arc<Self> {
        Arc::new(Self {
            mode: Mutex::new(mode),
            asked: Mutex::default(),
            running: AtomicUsize::new(0),
            most_running: AtomicUsize::new(0),
            release: Mutex::new(false),
            bytes: fixture(),
        })
    }
    fn set(&self, mode: Mode) {
        *self.mode.lock().unwrap() = mode;
    }
    fn asked(&self) -> usize {
        self.asked.lock().unwrap().len()
    }
}

impl TileBytes for Scripted {
    fn tile(&self, id: TileId) -> Result<Option<Vec<u8>>, String> {
        self.asked.lock().unwrap().push(id);
        let now = self.running.fetch_add(1, Ordering::SeqCst) + 1;
        self.most_running.fetch_max(now, Ordering::SeqCst);
        let mode = *self.mode.lock().unwrap();
        let answer = match mode {
            Mode::Answer => Ok(Some(self.bytes.clone())),
            Mode::Missing => Ok(None),
            Mode::Fail => Err("the network is down".to_owned()),
            Mode::Panic => panic!("a source that panics"),
            Mode::Hang => {
                while !*self.release.lock().unwrap() {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Ok(Some(self.bytes.clone()))
            }
        };
        self.running.fetch_sub(1, Ordering::SeqCst);
        answer
    }
    fn zoom_range(&self) -> (u8, u8) {
        (0, 12)
    }
    fn attribution(&self) -> Option<String> {
        Some("© OpenStreetMap contributors".into())
    }
}

fn limits(timeout_ms: u64, retry_ms: u64, most_kept: usize) -> Limits {
    Limits {
        load_timeout: Duration::from_millis(timeout_ms),
        retry_after: Duration::from_millis(retry_ms),
        most_kept,
    }
}

fn surface(source: &Arc<Scripted>, limits: Limits, zoom: f64) -> MapSurface {
    MapSurface::new(
        "map",
        MapViewLabels::default(),
        Arc::new(Nothing),
        Some(source.clone()),
        MapPosition::new(7.44, 46.95, zoom),
    )
    .with_limits(limits)
}

fn until(what: &str, mut ready: impl FnMut() -> bool) {
    let limit = Instant::now() + Duration::from_secs(10);
    while !ready() {
        assert!(Instant::now() < limit, "gave up waiting: {what}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Draw (which asks for the tiles) again and again until `ready`.
fn draw_until(s: &MapSurface, what: &str, mut ready: impl FnMut() -> bool) {
    until(what, || {
        let _ = s.build(600.0, 400.0);
        ready()
    });
}

#[test]
fn a_source_that_panics_leaves_the_tile_empty_and_the_map_alive() {
    let _g = serial();
    let source = Scripted::new(Mode::Panic);
    let s = surface(&source, limits(5000, 60_000, 256), 0.0);
    let _ = s.build(100.0, 100.0);
    until("the worker asked", || source.asked() > 0);
    std::thread::sleep(Duration::from_millis(100));
    let before = source.asked();
    let _ = s.build(100.0, 100.0);
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(s.tiles_kept(), 0, "no tile, an empty one");
    assert_eq!(
        source.asked(),
        before,
        "a failed tile is not asked for in the next draw"
    );
}

#[test]
fn at_most_four_tiles_are_fetched_at_once_and_a_hanging_source_gives_up_its_places_after_the_time()
{
    let _g = serial();
    let source = Scripted::new(Mode::Hang);
    let s = surface(&source, limits(150, 60_000, 256), 5.0);
    let _ = s.build(900.0, 700.0);
    until("four asked", || source.asked() >= MOST_LOADING);
    std::thread::sleep(Duration::from_millis(60));
    assert_eq!(source.asked(), MOST_LOADING, "no fifth while four hang");
    // after the time the four places are free: the next draw asks for others (bounded by the workers alive)
    std::thread::sleep(Duration::from_millis(200));
    draw_until(&s, "the places came back", || source.asked() > MOST_LOADING);
    *source.release.lock().unwrap() = true;
    until("all workers come back", || {
        source.running.load(Ordering::SeqCst) == 0
    });
}

#[test]
fn a_tile_that_is_no_longer_wanted_gives_up_its_place() {
    let _g = serial();
    let source = Scripted::new(Mode::Hang);
    let s = surface(&source, limits(60_000, 60_000, 256), 5.0);
    let _ = s.build(900.0, 700.0);
    until("four asked", || source.asked() >= MOST_LOADING);
    let first: HashSet<TileId> = source.asked.lock().unwrap().iter().copied().collect();
    // the view goes far away: the four places are free at once, without waiting for the time limit
    s.go_to(MapPosition::new(-120.0, -33.0, 5.0));
    draw_until(&s, "the new view asked", || {
        source
            .asked
            .lock()
            .unwrap()
            .iter()
            .any(|t| !first.contains(t))
    });
    *source.release.lock().unwrap() = true;
}

#[test]
fn a_failure_is_not_kept_for_good_it_is_asked_again_after_the_time() {
    let _g = serial();
    let source = Scripted::new(Mode::Fail);
    let s = surface(&source, limits(5000, 700, 256), 0.0);
    let _ = s.build(100.0, 100.0);
    until("asked", || source.asked() > 0);
    let once = source.asked();
    std::thread::sleep(Duration::from_millis(60));
    let _ = s.build(100.0, 100.0);
    assert_eq!(source.asked(), once, "not again at once");
    source.set(Mode::Answer);
    std::thread::sleep(Duration::from_millis(750));
    draw_until(&s, "the tile arrived after the retry", || {
        s.tiles_kept() > 0
    });
}

#[test]
fn no_such_tile_is_kept_for_good() {
    let _g = serial();
    let source = Scripted::new(Mode::Missing);
    let s = surface(&source, limits(5000, 50, 256), 0.0);
    draw_until(&s, "kept", || s.tiles_kept() > 0);
    let asked = source.asked();
    std::thread::sleep(Duration::from_millis(250));
    let _ = s.build(100.0, 100.0);
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(
        source.asked(),
        asked,
        "an answer of 'none' is not asked again"
    );
}

#[test]
fn the_oldest_tiles_go_when_there_are_too_many() {
    let _g = serial();
    let source = Scripted::new(Mode::Answer);
    let s = surface(&source, limits(5000, 60_000, 3), 4.0);
    // a view of many tiles, then others elsewhere: never more than three are kept
    for lon in [0.0, 40.0, 80.0, 120.0] {
        s.go_to(MapPosition::new(lon, 30.0, 4.0));
        let wanted = source.asked() + 1;
        draw_until(&s, "something new arrived", || source.asked() >= wanted);
        std::thread::sleep(Duration::from_millis(50));
        assert!(s.tiles_kept() <= 3, "{} kept", s.tiles_kept());
    }
    assert!(source.asked() > 3, "more were fetched than are kept");
    assert!(s.tiles_kept() > 0);
}

#[test]
fn the_source_names_who_made_the_data_and_the_map_shows_it() {
    let _g = serial();
    let source = Scripted::new(Mode::Missing);
    let s = surface(&source, limits(5000, 60_000, 8), 2.0);
    let tree = common::layout(s.build(600.0, 400.0), 600.0, 400.0);
    let node = tree
        .query_by_id("map-attribution")
        .expect("the attribution is shown");
    let b = tree.get_absolute_bounds(node).expect("bounds");
    assert!(
        b.x + b.width <= 600.5 && b.y + b.height <= 400.5,
        "inside the view"
    );
    assert!(b.width > 20.0 && b.height > 6.0);
}
