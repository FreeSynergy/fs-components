//! Hard data drawn on a real texture: it must not panic and must show what happened.

mod common;

use common::{background, out_dir, render, view};

fn doc(mark: &str, rows: &str, enc: &str) -> String {
    format!(
        r#"{{"title":"Edge case","data":{{"values":{rows}}},"mark":"{mark}","encoding":{{{enc}}}}}"#
    )
}

const CAT: &str = r#""x":{"field":"k","type":"nominal"},"y":{"field":"v","type":"quantitative"}"#;
const TIME: &str = r#""x":{"field":"d","type":"temporal"},"y":{"field":"v","type":"quantitative"}"#;

fn ink_of(json: &str, name: &str, w: u32, h: u32, dark: bool) -> usize {
    let (dir, _tmp) = out_dir(name);
    let pic = render(&view(json), w, h, dark, &dir, name);
    pic.ink([0.0, 0.0, w as f32, h as f32], pic.bg)
}

#[test]
fn empty_one_point_and_equal_values_draw() {
    assert!(ink_of(&doc("bar", "[]", CAT), "edge_empty", 360, 220, false) > 200);
    let one = r#"[{"k":"a","v":4,"d":"2024-03-10"}]"#;
    assert!(ink_of(&doc("line", one, TIME), "edge_one_line", 360, 220, false) > 300);
    assert!(ink_of(&doc("bar", one, CAT), "edge_one_bar", 360, 220, false) > 1500);
    let eq = r#"[{"k":"a","v":5},{"k":"b","v":5},{"k":"c","v":5}]"#;
    assert!(ink_of(&doc("bar", eq, CAT), "edge_equal", 360, 220, false) > 1500);
}

#[test]
fn broken_lines_and_dropped_bars_are_visible_with_their_note() {
    let rows = r#"[{"d":"2024-03-01","v":1},{"d":"2024-03-02","v":2},{"d":"2024-03-03","v":null},
        {"d":"2024-03-04","v":"NaN"},{"d":"2024-03-05","v":5},{"d":"2024-03-06","v":6}]"#;
    let (dir, _tmp) = out_dir("edge_nulls");
    let v = view(&doc("line", rows, TIME));
    let pic = render(&v, 460, 300, false, &dir, "edge_nulls");
    let scene = v.scene(460.0, 300.0);
    // the note sits under the chart and has ink
    let note_y = 300.0 - 5.0 - 3.0 * 15.0;
    assert!(
        pic.ink([0.0, note_y, 460.0, 45.0], background()) > 200,
        "the note is drawn"
    );
    // the gap between 2024-03-02 and 2024-03-05: no line colour between the runs
    let paths: Vec<_> = scene
        .items
        .iter()
        .filter_map(|i| {
            if let fs_components_charts::scene::Item::Path { points, .. } = i {
                Some(points.clone())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(paths.len(), 2);
    let (a, b) = (paths[0][1], paths[1][0]);
    let mid = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
    assert!(
        pic.ink([mid[0] - 2.0, mid[1] - 8.0, 4.0, 16.0], background()) < 40,
        "no line bridges the gap"
    );
}

#[test]
fn huge_unicode_long_and_many_draw_without_trouble() {
    let huge = r#"[{"k":"a","v":1.7e308},{"k":"b","v":-1.7e308},{"k":"c","v":1e-300}]"#;
    assert!(ink_of(&doc("bar", huge, CAT), "edge_huge", 400, 260, false) > 1500);
    let uni = r#"[{"k":"日本語","v":3},{"k":"Ünïcödé","v":5},{"k":"😀 emoji","v":2},{"k":"עברית","v":4}]"#;
    assert!(ink_of(&doc("bar", uni, CAT), "edge_unicode", 420, 260, false) > 1500);
    let long = format!(
        r#"[{{"k":"{}","v":3}},{{"k":"short","v":5}}]"#,
        "very long label ".repeat(20)
    );
    assert!(ink_of(&doc("bar", &long, CAT), "edge_long", 420, 300, false) > 1500);
    let many: Vec<String> = (0..300)
        .map(|i| format!(r#"{{"k":"item {i}","v":{}}}"#, (i * 37) % 101 + 1))
        .collect();
    assert!(
        ink_of(
            &doc("bar", &format!("[{}]", many.join(",")), CAT),
            "edge_many",
            700,
            300,
            false
        ) > 3000
    );
}

#[test]
fn dates_across_dst_and_a_hundred_thousand_points_draw() {
    let dst = r#"[{"d":"2024-03-29T12:00+01:00","v":1},{"d":"2024-03-30T12:00+01:00","v":2},{"d":"2024-03-31T12:00+02:00","v":3},{"d":"2024-04-01T12:00+02:00","v":2}]"#;
    assert!(ink_of(&doc("line", dst, TIME), "edge_dst", 460, 280, false) > 600);
    let rows: Vec<String> = (0..100_000)
        .map(|i| {
            format!(
                r#"{{"x":{},"y":{},"g":"g{}"}}"#,
                i % 1000,
                (i * 7919) % 1013,
                i % 4
            )
        })
        .collect();
    let json = doc(
        "point",
        &format!("[{}]", rows.join(",")),
        r#""x":{"field":"x","type":"quantitative"},"y":{"field":"y","type":"quantitative"},"color":{"field":"g","type":"nominal"}"#,
    );
    let t = std::time::Instant::now();
    let ink = ink_of(&json, "edge_100k", 600, 400, false);
    eprintln!(
        "100000 points: parse, layout and draw offscreen in {:?}",
        t.elapsed()
    );
    assert!(ink > 50_000, "a dense cloud covers the plot: {ink}");
    assert!(t.elapsed().as_secs() < 60);
}

#[test]
fn the_dark_theme_gets_its_own_colours() {
    let json = doc(
        "bar",
        r#"[{"k":"a","v":3},{"k":"b","v":5},{"k":"c","v":2}]"#,
        CAT,
    );
    let (dir, _tmp) = out_dir("edge_dark");
    let pic = render(&view(&json), 360, 240, true, &dir, "edge_dark");
    assert!(common::near(pic.pixel(2.0, 2.0), pic.bg, 4), "dark surface");
    assert!(pic.ink([0.0, 0.0, 360.0, 240.0], pic.bg) > 3000);
}
