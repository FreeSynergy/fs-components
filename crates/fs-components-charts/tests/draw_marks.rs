//! Every mark kind drawn offscreen; the pixels are checked against the layout and the theme.

mod common;

use common::{background, near, out_dir, render, to_u8, view};
use fs_components_charts::scene::{Item, Paint};
use fs_components_charts::view::Look;

fn series_colour(i: usize) -> [u8; 3] {
    to_u8(Look::current().colour(Paint::Series(i)))
}

const LINE: &str = r#"{
  "title": "Visitors per day",
  "description": "Visitors rise over the week",
  "data": {"values": [
    {"d": "2024-03-04", "v": 120}, {"d": "2024-03-05", "v": 180}, {"d": "2024-03-06", "v": 150},
    {"d": "2024-03-07", "v": 260}, {"d": "2024-03-08", "v": 310}, {"d": "2024-03-09", "v": 90}, {"d": "2024-03-10", "v": 60}]},
  "mark": "line",
  "encoding": {"x": {"field": "d", "type": "temporal", "axis": {"title": "Day"}}, "y": {"field": "v", "type": "quantitative", "axis": {"title": "Visitors"}}}
}"#;

#[test]
fn a_line_over_dates_passes_through_its_points() {
    let (dir, _tmp) = out_dir("line");
    let v = view(LINE);
    let pic = render(&v, 520, 320, false, &dir, "line");
    let scene = v.scene(520.0, 320.0);
    let path = scene
        .items
        .iter()
        .find_map(|i| {
            if let Item::Path {
                points,
                paint: Paint::Series(0),
                ..
            } = i
            {
                Some(points)
            } else {
                None
            }
        })
        .expect("a line path");
    assert_eq!(path.len(), 7);
    // every vertex lies on drawn line colour
    for p in &path[1..path.len() - 1] {
        let hit = (-2..=2).any(|dy| near(pic.pixel(p[0], p[1] + dy as f32), series_colour(0), 40));
        assert!(hit, "no line pixel near vertex {p:?}");
    }
    // x grows with the date: seven days over the plot width with equal steps
    let step = path[1][0] - path[0][0];
    assert!(
        path.windows(2)
            .all(|w| ((w[1][0] - w[0][0]) - step).abs() < 0.01)
    );
}

const AREA: &str = r#"{
  "data": {"values": [
    {"m": "Jan", "k": "a", "v": 3}, {"m": "Feb", "k": "a", "v": 5}, {"m": "Mar", "k": "a", "v": 4},
    {"m": "Jan", "k": "b", "v": 2}, {"m": "Feb", "k": "b", "v": 2}, {"m": "Mar", "k": "b", "v": 6}]},
  "mark": "area",
  "encoding": {"x": {"field": "m", "type": "ordinal", "sort": null}, "y": {"field": "v", "type": "quantitative"}, "color": {"field": "k", "type": "nominal"}}
}"#;

#[test]
fn stacked_areas_put_series_zero_on_top_and_fill_between() {
    let (dir, _tmp) = out_dir("area");
    let v = view(AREA);
    let pic = render(&v, 520, 320, false, &dir, "area");
    let scene = v.scene(520.0, 320.0);
    let polys: Vec<_> = scene
        .items
        .iter()
        .filter_map(|i| {
            if let Item::Polygon {
                points,
                paint: Paint::Series(s),
                ..
            } = i
            {
                Some((*s, points.clone()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(polys.len(), 2);
    let top_y = |s: usize| polys.iter().find(|p| p.0 == s).unwrap().1[0][1];
    assert!(top_y(0) < top_y(1), "series 0 is on top (smaller y)");
    // inside series 1's polygon, away from its edge, the pixel carries colour of series 1 (blended)
    let p = &polys.iter().find(|p| p.0 == 1).unwrap().1;
    let (mx, my) = (
        (p[0][0] + p[1][0]) / 2.0,
        (p[0][1] + p[p.len() - 1][1]) / 2.0 + 2.0,
    );
    assert!(
        !near(pic.pixel(mx, my), background(), 20),
        "area interior must be filled"
    );
}

const POINTS: &str = r#"{
  "data": {"values": [
    {"x": 1, "y": 2, "g": "a"}, {"x": 2, "y": 4, "g": "a"}, {"x": 3, "y": 3, "g": "b"},
    {"x": 4, "y": 6, "g": "b"}, {"x": 5, "y": 5, "g": "c"}, {"x": 6, "y": 8, "g": "c"}]},
  "mark": "point",
  "encoding": {"x": {"field": "x", "type": "quantitative"}, "y": {"field": "y", "type": "quantitative"}, "color": {"field": "g", "type": "nominal", "title": "Group"}}
}"#;

#[test]
fn points_have_a_symbol_per_series_and_a_legend() {
    let (dir, _tmp) = out_dir("point");
    let v = view(POINTS);
    let pic = render(&v, 520, 320, false, &dir, "point");
    let scene = v.scene(520.0, 320.0);
    let markers: Vec<_> = scene
        .items
        .iter()
        .filter_map(|i| {
            if let Item::Marker {
                at, shape, paint, ..
            } = i
            {
                Some((*at, *shape, *paint))
            } else {
                None
            }
        })
        .collect();
    // six data points and three legend symbols
    assert_eq!(markers.len(), 9);
    use std::collections::HashSet;
    let shapes: HashSet<_> = markers.iter().map(|m| format!("{:?}", m.1)).collect();
    assert_eq!(shapes.len(), 3, "three series, three shapes: {shapes:?}");
    // the first point is an open circle: ink on the ring, the surface in the middle
    let (at, _, _) = markers[0];
    let bg = background();
    assert!(
        pic.ink([at[0] - 4.0, at[1] - 4.0, 8.0, 8.0], bg) >= 8,
        "outline of the first point"
    );
    assert!(
        !near(pic.pixel(at[0], at[1]), series_colour(0), 60),
        "an open circle is not filled in the middle"
    );
}

const PIE: &str = r#"{
  "data": {"values": [{"k": "Rent", "v": 40}, {"k": "Food", "v": 25}, {"k": "Travel", "v": 20}, {"k": "Other", "v": 15}]},
  "mark": {"type": "arc", "innerRadius": 50},
  "encoding": {"theta": {"field": "v", "type": "quantitative"}, "color": {"field": "k", "type": "nominal"}}
}"#;

#[test]
fn a_donut_has_proportional_sectors_and_a_hole() {
    let (dir, _tmp) = out_dir("arc");
    let v = view(PIE);
    let pic = render(&v, 420, 300, false, &dir, "arc");
    let scene = v.scene(420.0, 300.0);
    let secs: Vec<_> = scene
        .items
        .iter()
        .filter_map(|i| {
            if let Item::Sector {
                start,
                end,
                paint: Paint::Series(s),
                centre,
                inner,
                outer,
            } = i
            {
                Some((*s, end - start, *centre, *inner, *outer))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(secs.len(), 4);
    let total: f32 = secs.iter().map(|s| s.1).sum();
    assert!((total - std::f32::consts::TAU).abs() < 1e-4);
    // categories sort ascending: Food, Other, Rent, Travel -> 25, 15, 40, 20 of 100
    let by_series = |i: usize| secs.iter().find(|s| s.0 == i).unwrap().1 / std::f32::consts::TAU;
    for (i, want) in [0.25, 0.15, 0.40, 0.20].into_iter().enumerate() {
        assert!(
            (by_series(i) - want).abs() < 1e-4,
            "series {i}: {} vs {want}",
            by_series(i)
        );
    }
    let (c, inner) = (secs[0].2, secs[0].3);
    assert!(
        near(pic.pixel(c[0], c[1]), background(), 8),
        "the hole shows the surface"
    );
    assert!(
        !near(pic.pixel(c[0] + inner + 8.0, c[1]), background(), 20),
        "the ring is filled"
    );
}

#[test]
fn stacked_and_normalised_bars_with_a_legend() {
    let (dir, _tmp) = out_dir("stack");
    let json = r#"{"data":{"values":[
        {"q":"Q1","k":"a","v":3},{"q":"Q1","k":"b","v":1},{"q":"Q2","k":"a","v":2},{"q":"Q2","k":"b","v":6}]},
      "mark":"bar","encoding":{"x":{"field":"q","type":"nominal"},"y":{"field":"v","type":"quantitative","stack":"normalize"},"color":{"field":"k","type":"nominal"}}}"#;
    let v = view(json);
    let pic = render(&v, 420, 300, false, &dir, "stack_normalize");
    let scene = v.scene(420.0, 300.0);
    let rects: Vec<_> = scene
        .items
        .iter()
        .filter_map(|i| {
            if let Item::Rect {
                x,
                y,
                w,
                h,
                paint: Paint::Series(s),
                ..
            } = i
            {
                Some((*s, [*x, *y, *w, *h]))
            } else {
                None
            }
        })
        .collect();
    // 4 bars and 2 legend swatches
    assert_eq!(rects.len(), 6);
    // Q1: a is 3/4 of the height; Q2: b is 6/8
    let q1: Vec<_> = rects
        .iter()
        .filter(|r| r.1[2] > 20.0 && r.1[0] < 200.0)
        .collect();
    let total = q1.iter().map(|r| r.1[3]).sum::<f32>();
    let a = q1.iter().find(|r| r.0 == 0).unwrap().1[3];
    assert!((a / total - 0.75).abs() < 1e-3, "{a} of {total}");
    let y_of = |series: usize| q1.iter().find(|r| r.0 == series).unwrap().1[1];
    assert!(y_of(0) < y_of(1), "series 0 is on top of the stack");
    assert!(
        (total - scene.plot[3]).abs() < 0.01,
        "a normalised stack fills the plot height"
    );
    assert!(pic.ink(scene.plot, background()) > 2000);
}
