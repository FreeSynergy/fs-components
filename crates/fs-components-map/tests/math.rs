//! `map-view`: the arithmetic, with no graphics. Extreme values must give numbers on the map.

use fs_components_map::{
    MAX_LAT, MAX_ZOOM, MOST_TILES, MapCommand, MapPosition, TILE_SIZE, command_for_key,
    needed_tiles, project, tile_zoom, unproject, world_size, wrap_lon,
};
use fs_components_widgets::{Key, KeyEvent};

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

#[test]
fn the_world_is_one_tile_at_zoom_zero() {
    assert_eq!(world_size(0.0), TILE_SIZE);
    let (x, y) = project(0.0, 0.0, 0.0);
    assert!(near(x, 128.0) && near(y, 128.0));
    let (x, y) = project(-180.0, MAX_LAT, 0.0);
    assert!(near(x, 0.0) && near(y, 0.0));
}

#[test]
fn project_and_unproject_are_inverse() {
    for (lon, lat, zoom) in [(13.4, 52.5, 10.0), (-122.4, 37.8, 5.5), (179.9, -60.0, 3.0)] {
        let (x, y) = project(lon, lat, zoom);
        let (back_lon, back_lat) = unproject(x, y, zoom);
        assert!(near(back_lon, lon) && near(back_lat, lat), "{lon},{lat}");
    }
}

#[test]
fn berlin_is_in_the_known_tile() {
    let berlin = MapPosition::new(13.3777, 52.5163, 10.0);
    let set = needed_tiles(berlin, 100.0, 100.0, 0, 14);
    let first = set.tiles[0].id;
    assert_eq!((first.z, first.x, first.y), (10, 550, 335));
}

#[test]
fn bad_numbers_become_numbers_on_the_map() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1e300, -1e300] {
        let p = MapPosition::new(bad, bad, bad);
        assert!((-180.0..180.0).contains(&p.lon()), "{bad}");
        assert!(p.lat().abs() <= MAX_LAT && p.zoom() <= MAX_ZOOM && p.zoom() >= 0.0);
        let q = p
            .panned_by(bad, bad)
            .zoomed_by(bad)
            .zoomed_at(bad, bad, bad, bad, bad);
        assert!(q.lon().is_finite() && q.lat().is_finite() && q.zoom().is_finite());
        let (x, y) = q.to_screen(bad, bad, bad, bad);
        assert!(x.is_finite() && y.is_finite(), "{bad}");
        let _ = q.from_screen(bad, bad, bad, bad);
        assert!(project(bad, bad, bad).0.is_finite());
        assert!(unproject(bad, bad, bad).1.is_finite());
    }
}

#[test]
fn the_poles_stop_the_pan_and_the_longitude_wraps() {
    let mut p = MapPosition::new(0.0, 80.0, 4.0);
    for _ in 0..200 {
        p = p.panned_by(0.0, -500.0);
    }
    assert!(p.lat() <= MAX_LAT && p.lat() > 84.0);
    let mut q = MapPosition::new(179.0, 0.0, 3.0);
    q = q.panned_by(5000.0, 0.0);
    assert!((-180.0..180.0).contains(&q.lon()));
    assert!(
        q.lon() < 0.0,
        "went east across the antimeridian: {}",
        q.lon()
    );
    assert!(near(wrap_lon(190.0), -170.0) && near(wrap_lon(-190.0), 170.0));
    assert!(near(wrap_lon(180.0), -180.0) && near(wrap_lon(540.0), -180.0));
    assert!((-180.0..180.0).contains(&wrap_lon(-1e-20)));
}

#[test]
fn a_place_across_the_antimeridian_is_drawn_beside_the_centre() {
    let p = MapPosition::new(179.5, 0.0, 4.0);
    let (x, _) = p.to_screen(-179.5, 0.0, 800.0, 600.0);
    let (centre, _) = p.to_screen(179.5, 0.0, 800.0, 600.0);
    assert!(near(centre, 400.0));
    assert!(x > centre && x < centre + 100.0, "{x}");
}

#[test]
fn zoom_stays_in_limits_and_a_zoom_at_a_point_keeps_the_point() {
    let p = MapPosition::new(10.0, 50.0, 5.0);
    assert_eq!(p.zoomed_by(100.0).zoom(), MAX_ZOOM);
    assert_eq!(p.zoomed_by(-100.0).zoom(), 0.0);
    let before = p.from_screen(600.0, 100.0, 800.0, 600.0);
    let after = p
        .zoomed_at(2.0, 600.0, 100.0, 800.0, 600.0)
        .from_screen(600.0, 100.0, 800.0, 600.0);
    assert!(
        near(before.0, after.0) && near(before.1, after.1),
        "{before:?} {after:?}"
    );
}

#[test]
fn fitting_a_box_shows_it_whole() {
    let p = MapPosition::fitting((5.0, 45.0, 15.0, 55.0), 800.0, 600.0, 18.0);
    let (x0, y0) = p.to_screen(5.0, 55.0, 800.0, 600.0);
    let (x1, y1) = p.to_screen(15.0, 45.0, 800.0, 600.0);
    assert!(
        x0 >= -1.0 && y0 >= -1.0 && x1 <= 801.0 && y1 <= 601.0,
        "{x0} {y0} {x1} {y1}"
    );
    let across = MapPosition::fitting((170.0, -10.0, -170.0, 10.0), 800.0, 600.0, 18.0);
    assert!(across.lon().abs() > 175.0, "{}", across.lon());
    let point = MapPosition::fitting((1.0, 1.0, 1.0, 1.0), 800.0, 600.0, 15.0);
    assert_eq!(point.zoom(), 15.0);
    let _ = MapPosition::fitting((f64::NAN, 1.0, f64::INFINITY, 1.0), 0.0, f64::NAN, 15.0);
}

#[test]
fn tiles_for_a_view_cover_it_and_wrap() {
    let p = MapPosition::new(0.0, 0.0, 2.0);
    let set = needed_tiles(p, 1400.0, 600.0, 0, 14);
    assert!(!set.truncated);
    // The world is 1024 px wide: 1400 px shows part of it twice.
    let xs: Vec<u32> = set.tiles.iter().map(|t| t.id.x).collect();
    assert!(xs.iter().all(|x| *x < 4));
    let lefts: Vec<f64> = set.tiles.iter().map(|t| t.left).collect();
    assert!(lefts.iter().any(|l| *l < 0.0) && lefts.iter().any(|l| *l > 800.0));
    // Every pixel of the view is covered by some tile.
    for (px, py) in [(0.0, 0.0), (1399.0, 599.0), (700.0, 300.0), (50.0, 590.0)] {
        assert!(
            set.tiles.iter().any(|t| px >= t.left
                && px < t.left + t.size
                && py >= t.top
                && py < t.top + t.size),
            "{px},{py}"
        );
    }
}

#[test]
fn rows_beyond_the_world_are_left_out() {
    let p = MapPosition::new(0.0, MAX_LAT, 3.0);
    let set = needed_tiles(p, 600.0, 600.0, 0, 14);
    assert!(set.tiles.iter().all(|t| t.id.y < 8));
    assert!(set.tiles.iter().all(|t| t.id.y <= 1));
}

#[test]
fn the_number_of_tiles_is_capped_for_any_size_and_zoom() {
    let started = std::time::Instant::now();
    for (w, h, zoom, min, max) in [
        (1e9, 1e9, 20.0, 0, 22),
        (50_000.0, 50_000.0, 12.0, 0, 14),
        (4000.0, 3000.0, 0.0, 6, 14),
        (f64::MAX, f64::MAX, 22.0, 0, 22),
    ] {
        let set = needed_tiles(MapPosition::new(10.0, 10.0, zoom), w, h, min, max);
        assert!(set.tiles.len() <= MOST_TILES, "{w}x{h}");
        assert!(set.truncated || set.tiles.len() < MOST_TILES);
    }
    assert!(started.elapsed().as_secs() < 2);
}

#[test]
fn no_size_needs_no_tiles_and_extreme_zoom_does_not_panic() {
    let p = MapPosition::new(0.0, 0.0, 5.0);
    for (w, h) in [(0.0, 0.0), (0.0, 100.0), (-5.0, 100.0), (f64::NAN, 1.0)] {
        assert_eq!(
            needed_tiles(p, w, h, 0, 14).tiles,
            [] as [fs_components_map::NeededTile; 0]
        );
    }
    assert_eq!(tile_zoom(22.0, 0, 3), 3);
    assert_eq!(tile_zoom(0.4, 5, 9), 5);
    assert_eq!(tile_zoom(f64::NAN, 0, 9), 0);
    assert_eq!(tile_zoom(7.9, 9, 5), 7);
    let deep = needed_tiles(MapPosition::new(1.0, 1.0, 22.0), 800.0, 600.0, 0, 2);
    assert!(deep.tiles.len() <= 4 && deep.tiles.iter().all(|t| t.id.z <= 2));
}

#[test]
fn keys_and_buttons_are_the_same_values() {
    let key = |k| command_for_key(KeyEvent::new(k));
    assert_eq!(key(Key::Left), Some(MapCommand::Pan(-96.0, 0.0)));
    assert_eq!(key(Key::Down), Some(MapCommand::Pan(0.0, 96.0)));
    assert_eq!(key(Key::PageUp), Some(MapCommand::Zoom(1.0)));
    assert_eq!(key(Key::Other(189)), Some(MapCommand::Zoom(-1.0)));
    assert_eq!(key(Key::Other(65)), None);
    let mut ctrl = KeyEvent::new(Key::Left);
    ctrl.ctrl = true;
    assert_eq!(command_for_key(ctrl), None);
    let p = MapPosition::new(0.0, 0.0, 4.0);
    assert!(MapCommand::Pan(96.0, 0.0).apply(p, 800.0, 600.0).lon() > 0.0);
    assert_eq!(MapCommand::Zoom(1.0).apply(p, 0.0, 0.0).zoom(), 5.0);
}
