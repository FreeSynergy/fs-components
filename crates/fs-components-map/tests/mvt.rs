//! Vector tiles into the neutral model, and the surface that fetches them without blocking the draw.
//!
//! Pattern: none (tests). The fixture tile is written with the `mvt` encoder, so the decoder is checked against bytes
//! it did not make.

mod common;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::serial;
use fs_components_app::Surface;
use fs_components_map::{
    MapData, MapPosition, MapSurface, MapViewLabels, Marker, TileBytes, TileId, decode_tile,
};
use mvt::{GeomEncoder, GeomType, Tile};

fn fixture() -> Vec<u8> {
    let mut tile = Tile::new(4096);
    let water = tile.create_layer("water");
    let lake = GeomEncoder::new(GeomType::Polygon)
        .point(100.0, 100.0)
        .and_then(|g| g.point(900.0, 100.0))
        .and_then(|g| g.point(900.0, 900.0))
        .and_then(|g| g.point(100.0, 900.0))
        .and_then(GeomEncoder::complete)
        .and_then(GeomEncoder::encode)
        .unwrap();
    let water = water.into_feature(lake).into_layer();
    tile.add_layer(water).unwrap();
    let roads = tile.create_layer("roads");
    let road = GeomEncoder::new(GeomType::Linestring)
        .point(0.0, 2000.0)
        .and_then(|g| g.point(4000.0, 2100.0))
        .and_then(GeomEncoder::encode)
        .unwrap();
    let roads = roads.into_feature(road).into_layer();
    tile.add_layer(roads).unwrap();
    let places = tile.create_layer("places");
    let town = GeomEncoder::new(GeomType::Point)
        .point(2048.0, 2048.0)
        .and_then(GeomEncoder::encode)
        .unwrap();
    let mut f = places.into_feature(town);
    f.add_tag_string("name", "Bern");
    tile.add_layer(f.into_layer()).unwrap();
    tile.to_bytes().unwrap()
}

#[test]
fn a_vector_tile_becomes_areas_lines_and_labels() {
    let g = decode_tile(&fixture()).unwrap();
    assert!((g.extent - 4096.0).abs() < f32::EPSILON);
    assert_eq!(g.areas.len(), 1);
    assert_eq!(g.lines.len(), 1);
    assert_eq!(g.labels.len(), 1);
    assert_eq!(&*g.labels[0].text, "Bern");
}

#[test]
fn bytes_that_are_no_tile_are_an_error_not_a_panic() {
    assert!(decode_tile(&[0xff, 0xff, 0xff, 0xff, 0x01]).is_err());
}

struct Pins(Mutex<Vec<String>>);
impl MapData for Pins {
    fn markers(&self) -> Vec<Marker> {
        vec![Marker::new("a", 7.44, 46.95).labelled("Bern")]
    }
    fn picked(&self, key: &str) {
        self.0.lock().unwrap().push(key.to_owned());
    }
}

struct Slow(Mutex<Vec<TileId>>, Vec<u8>);
impl TileBytes for Slow {
    fn tile(&self, id: TileId) -> Result<Option<Vec<u8>>, String> {
        std::thread::sleep(Duration::from_millis(30));
        self.0.lock().unwrap().push(id);
        Ok(Some(self.1.clone()))
    }
    fn zoom_range(&self) -> (u8, u8) {
        (0, 2)
    }
}

#[test]
fn the_surface_draws_at_once_and_keeps_the_tiles_that_arrive_later() {
    let _g = serial();
    let source = Arc::new(Slow(Mutex::default(), fixture()));
    let surface = MapSurface::new(
        "map",
        MapViewLabels::default(),
        Arc::new(Pins(Mutex::default())),
        Some(source.clone()),
        MapPosition::new(7.44, 46.95, 1.0),
    );
    let started = Instant::now();
    let _ = surface.build(600.0, 400.0);
    assert!(
        started.elapsed() < Duration::from_millis(25),
        "the draw does not wait for a tile"
    );
    let limit = Instant::now() + Duration::from_secs(10);
    while surface.tiles_kept() == 0 && Instant::now() < limit {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(surface.tiles_kept() > 0, "a tile arrived and was kept");
    surface.go_to(MapPosition::new(8.0, 47.0, 1.0));
    assert!((surface.position().lon() - 8.0).abs() < 1e-9);
    assert!(!source.0.lock().unwrap().is_empty());
}
