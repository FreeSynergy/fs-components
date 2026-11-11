//! What the loading tests share: a source that answers, hangs, fails or panics on demand, and a surface over it.
//!
//! Pattern: none (test support).
#![allow(dead_code)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use fs_components_app::Surface;
use fs_components_map::{
    Limits, MapData, MapPosition, MapSurface, MapViewLabels, Marker, TileBytes, TileId,
};
use mvt::{GeomEncoder, GeomType, Tile};

pub fn fixture() -> Vec<u8> {
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

pub struct Nothing;
impl MapData for Nothing {
    fn markers(&self) -> Vec<Marker> {
        Vec::new()
    }
    fn picked(&self, _: &str) {}
}

/// What a source does with a tile.
#[derive(Clone, Copy)]
pub enum Mode {
    Answer,
    Hang,
    Fail,
    Panic,
    Missing,
}

pub struct Scripted {
    pub mode: Mutex<Mode>,
    pub asked: Mutex<Vec<TileId>>,
    pub running: AtomicUsize,
    pub most_running: AtomicUsize,
    pub release: Mutex<bool>,
    pub bytes: Vec<u8>,
}

impl Scripted {
    pub fn new(mode: Mode) -> Arc<Self> {
        Arc::new(Self {
            mode: Mutex::new(mode),
            asked: Mutex::default(),
            running: AtomicUsize::new(0),
            most_running: AtomicUsize::new(0),
            release: Mutex::new(false),
            bytes: fixture(),
        })
    }
    pub fn set(&self, mode: Mode) {
        *self.mode.lock().unwrap() = mode;
    }
    pub fn asked(&self) -> usize {
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

pub fn limits(timeout_ms: u64, retry_ms: u64, most_kept: usize) -> Limits {
    Limits {
        load_timeout: Duration::from_millis(timeout_ms),
        retry_after: Duration::from_millis(retry_ms),
        most_kept,
        ..Limits::default()
    }
}

pub fn surface(source: &Arc<Scripted>, limits: Limits, zoom: f64) -> MapSurface {
    MapSurface::new(
        "map",
        MapViewLabels::default(),
        Arc::new(Nothing),
        Some(source.clone()),
        MapPosition::new(7.44, 46.95, zoom),
    )
    .with_limits(limits)
}

pub fn until(what: &str, mut ready: impl FnMut() -> bool) {
    let limit = Instant::now() + Duration::from_secs(10);
    while !ready() {
        assert!(Instant::now() < limit, "gave up waiting: {what}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Draw (which asks for the tiles) again and again until `ready`.
pub fn draw_until(s: &MapSurface, what: &str, mut ready: impl FnMut() -> bool) {
    until(what, || {
        let _ = s.build(600.0, 400.0);
        ready()
    });
}
