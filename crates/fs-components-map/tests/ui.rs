//! `map-view`: the real element drawn offscreen, pressed over the router a window uses, and timed.
//!
//! Pattern: Picture and input tests. No window is opened. A hang here is a deadlock.

mod common;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use common::{Input, background, draw, ink, layout, serial};
use fs_components_map::{
    Area, Dot, Label, Line, MapPosition, MapView, MapViewLabels, Marker, NeededTile, Paint,
    TileGeometry, TileId, Tone, needed_tiles,
};

fn tile_at(z: u8, x: u32, y: u32, left: f64, top: f64) -> NeededTile {
    NeededTile {
        id: TileId { z, x, y },
        left,
        top,
        size: 256.0,
    }
}

/// A tile with a lake (a hole in a field), a road, a town and a word.
fn sample() -> Arc<TileGeometry> {
    let mut g = TileGeometry::new(4096.0);
    g.areas.push(Area {
        rings: vec![
            vec![[0.0, 0.0], [4096.0, 0.0], [4096.0, 4096.0], [0.0, 4096.0]],
            vec![
                [1000.0, 1000.0],
                [1000.0, 2500.0],
                [2500.0, 2500.0],
                [2500.0, 1000.0],
            ],
        ],
        fill: Paint::Theme(Tone::Success, 0.5),
        outline: Some((Paint::tone(Tone::Border), 1.0)),
    });
    g.lines.push(Line {
        points: vec![[0.0, 3000.0], [2000.0, 3300.0], [4096.0, 3000.0]],
        paint: Paint::tone(Tone::Warning),
        width: 4.0,
    });
    g.dots.push(Dot {
        at: [3200.0, 800.0],
        paint: Paint::tone(Tone::Error),
        radius: 5.0,
    });
    g.labels.push(Label {
        at: [2048.0, 3700.0],
        text: "Roadside".into(),
        paint: Paint::tone(Tone::Text),
        size: 14.0,
    });
    Arc::new(g)
}

fn view(markers: Vec<Marker>) -> MapView {
    let p = MapPosition::new(0.0, 0.0, 0.0);
    MapView::new("map", p, MapViewLabels::default())
        .tiles(vec![(tile_at(0, 0, 0, 72.0, 0.0), sample())])
        .markers(markers)
}

#[test]
fn a_tile_is_drawn_with_its_hole_its_road_and_its_dot() {
    let _g = serial();
    let img = draw(
        view(vec![]).build(400.0, 256.0),
        400,
        256,
        "map_view_tile.png",
    );
    let bg = background(&img);
    // The field is painted, the hole in it is not (it shows the page).
    let field = ink(&img, (80, 8, 20, 20), bg);
    let hole = ink(&img, (72 + 80, 90, 20, 20), bg);
    assert!(field > 200, "field {field}");
    assert!(hole < 20, "hole {hole}");
    // The road crosses the lower part.
    assert!(ink(&img, (72 + 100, 130, 20, 40), bg) > 40);
}

#[test]
fn a_marked_marker_looks_different_and_a_chosen_one_can_be_pressed() {
    let _g = serial();
    let picked = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&picked);
    let markers = vec![
        Marker::new("home", 0.0, 0.0)
            .marked()
            .labelled("Must not appear"),
        Marker::new("pub", 30.0, 20.0)
            .labelled("The pub")
            .selected(),
    ];
    let map = view(markers).on_pick(Arc::new(move |key| {
        sink.lock().unwrap().push(key.to_string());
    }));
    let scene = map.scene(400.0, 256.0);
    assert_eq!(scene.markers.len(), 2);
    assert_eq!(
        scene.markers[0].marker.label, None,
        "a marked marker has no word"
    );
    let mut tree = layout(map.build(400.0, 256.0), 400.0, 256.0);
    let mut input = Input::new();
    input.click(&mut tree, "map-marker-pub");
    input.click(&mut tree, "map-marker-home");
    assert_eq!(
        *picked.lock().unwrap(),
        vec!["pub".to_string(), "home".to_string()]
    );
}

#[test]
fn keys_and_buttons_report_the_new_position() {
    let _g = serial();
    let seen = Arc::new(Mutex::new(Vec::<MapPosition>::new()));
    let sink = Arc::clone(&seen);
    let map = MapView::new(
        "map",
        MapPosition::new(10.0, 10.0, 5.0),
        MapViewLabels::default(),
    )
    .on_change(Arc::new(move |p| sink.lock().unwrap().push(p)));
    let mut tree = layout(map.build(500.0, 400.0), 500.0, 400.0);
    let mut input = Input::new();
    input.click(&mut tree, "map-zoom-in");
    input.click(&mut tree, "map-west");
    // Tab reaches the map and then a key pans it, in the order of a window.
    for _ in 0..6 {
        if seen.lock().unwrap().len() > 2 {
            break;
        }
        input.key(&mut tree, 9);
        input.key(&mut tree, 39);
    }
    let seen = seen.lock().unwrap();
    assert_eq!(seen[0].zoom(), 6.0);
    assert!(seen[1].lon() < 10.0);
    assert!(seen.len() > 2, "a key reached the map: {}", seen.len());
    assert!(seen[2].lon() > 10.0, "Right pans east");
}

#[test]
fn zero_sizes_and_bad_numbers_give_an_element_not_a_panic() {
    let _g = serial();
    for (w, h) in [
        (0.0, 0.0),
        (0.0, 300.0),
        (300.0, 0.0),
        (-4.0, -4.0),
        (f32::NAN, 100.0),
    ] {
        let map = view(vec![Marker::new("x", f64::NAN, f64::INFINITY)]);
        let _ = layout(map.build(w, h), w.max(1.0), h.max(1.0));
    }
    let far = MapView::new(
        "far",
        MapPosition::new(f64::NAN, 99.0, 99.0),
        MapViewLabels::default(),
    )
    .tiles(vec![(tile_at(0, 0, 0, f64::NAN, f64::INFINITY), sample())]);
    let _ = draw(far.build(200.0, 200.0), 200, 200, "map_view_far.png");
}

/// Many elements: the time to plan and to draw. Printed, not tight: it says what a view costs.
#[test]
fn many_elements_are_planned_and_drawn_in_a_measured_time() {
    let _g = serial();
    let mut g = TileGeometry::new(4096.0);
    for i in 0..4000u16 {
        let (x, y) = (f32::from(i % 63) * 64.0, f32::from(i / 63) * 64.0);
        g.areas.push(Area {
            rings: vec![vec![
                [x, y],
                [x + 50.0, y],
                [x + 50.0, y + 50.0],
                [x, y + 50.0],
            ]],
            fill: Paint::Theme(Tone::Info, 0.4),
            outline: None,
        });
        g.lines.push(Line {
            points: vec![[x, y], [x + 60.0, y + 30.0], [x + 10.0, y + 60.0]],
            paint: Paint::tone(Tone::Border),
            width: 1.0,
        });
        g.dots.push(Dot {
            at: [x + 5.0, y + 5.0],
            paint: Paint::tone(Tone::Primary),
            radius: 2.0,
        });
    }
    let g = Arc::new(g);
    let position = MapPosition::new(0.0, 0.0, 2.0);
    let started = Instant::now();
    let set = needed_tiles(position, 1200.0, 800.0, 0, 14);
    let tiles: Vec<_> = set.tiles.iter().map(|t| (*t, Arc::clone(&g))).collect();
    let map = MapView::new("big", position, MapViewLabels::default()).tiles(tiles);
    let scene = map.scene(1200.0, 800.0);
    let planned = started.elapsed();
    let visible = scene.drawn();
    let drawn_at = Instant::now();
    let _ = draw(map.build(1200.0, 800.0), 1200, 800, "map_view_many.png");
    let drawn = drawn_at.elapsed();
    eprintln!(
        "map_view many: {} tiles x {} primitives, {visible} drawn, {} left out by the budget; plan {planned:?}; build+layout+draw+read {drawn:?}",
        scene.tiles.len(),
        g.len(),
        scene.omitted
    );
    assert!(
        visible > 0 && scene.omitted > 0,
        "{visible} drawn, {} left out",
        scene.omitted
    );
    // Within Blinc's frame budget: what the plan chose is within what Blinc draws whole, so the
    // tiles nearest the centre are there, not the ones drawn first.
    assert!(visible <= 14_000, "{visible}");
    assert!(drawn.as_secs() < 60, "{drawn:?}");
}

struct Credited;
impl fs_components_map::TileBytes for Credited {
    fn tile(&self, _: TileId) -> Result<Option<Vec<u8>>, String> {
        Ok(None)
    }
    fn attribution(&self) -> Option<String> {
        Some("© OpenStreetMap contributors".into())
    }
}

struct NoMarkers;
impl fs_components_map::MapData for NoMarkers {
    fn markers(&self) -> Vec<Marker> {
        Vec::new()
    }
    fn picked(&self, _: &str) {}
}

#[test]
fn the_source_names_who_made_the_data_and_the_map_shows_it() {
    let _g = serial();
    let s = fs_components_map::MapSurface::new(
        "map",
        MapViewLabels::default(),
        Arc::new(NoMarkers),
        Some(Arc::new(Credited)),
        MapPosition::new(7.44, 46.95, 2.0),
    );
    let tree = layout(
        fs_components_app::Surface::build(&s, 600.0, 400.0),
        600.0,
        400.0,
    );
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
