//! More marks than Blinc's drawing surface holds: axes stay, the line keeps its shape, and what
//! cannot be drawn is counted in a note.

mod common;

use common::{out_dir, render, view};
use fs_components_charts::scene::Item;

fn series_doc(mark: &str, n: usize, enc_extra: &str) -> String {
    let rows: Vec<String> = (0..n)
        .map(|i| {
            format!(
                r#"{{"x":{i},"y":{:.3},"k":"c{i}"}}"#,
                (i as f64 * 0.01).sin() * 100.0 + (i % 7) as f64
            )
        })
        .collect();
    format!(
        r#"{{"title":"{mark} {n}","data":{{"values":[{}]}},"mark":"{mark}","encoding":{{"x":{{"field":"x","type":"quantitative"}},"y":{{"field":"y","type":"quantitative"}}{enc_extra}}}}}"#,
        rows.join(",")
    )
}

#[test]
fn a_line_of_100000_points_is_merged_per_pixel_and_complete() {
    let (dir, _tmp) = out_dir("lim_line");
    let v = view(&series_doc("line", 100_000, ""));
    let pic = render(&v, 600, 300, false, &dir, "lim_line");
    let scene = v.scene(600.0, 300.0);
    let pts: usize = scene
        .items
        .iter()
        .map(|i| {
            if let Item::Path { points, .. } = i {
                points.len()
            } else {
                0
            }
        })
        .sum();
    assert!(pts <= 4 * 600, "{pts} points for 600 pixels");
    assert_eq!(
        scene.omitted, 0,
        "merging per pixel loses nothing the eye can see"
    );
    // the right end of the plot has ink: the line is not cut off
    let p = scene.plot;
    assert!(pic.ink([p[0] + p[2] - 40.0, p[1], 38.0, p[3]], pic.bg) > 300);
    // axes survived: ink under the plot (tick labels)
    assert!(pic.ink([p[0], p[1] + p[3] + 4.0, p[2], 14.0], pic.bg) > 100);
}

#[test]
fn an_area_of_100000_points_draws_in_reasonable_time() {
    let (dir, _tmp) = out_dir("lim_area");
    let v = view(&series_doc("area", 100_000, ""));
    let t = std::time::Instant::now();
    let _ = render(&v, 600, 300, false, &dir, "lim_area");
    eprintln!("area of 100000 points drawn in {:?}", t.elapsed()); // it ends: that is the test
}

#[test]
fn thirty_thousand_bars_are_not_drawn_in_part_and_the_note_says_so() {
    let (dir, _tmp) = out_dir("lim_bars");
    let json = series_doc("bar", 30_000, "").replace(
        r#""x":{"field":"x","type":"quantitative"}"#,
        r#""x":{"field":"k","type":"nominal"}"#,
    );
    let v = view(&json);
    let pic = render(&v, 600, 300, false, &dir, "lim_bars");
    let scene = v.scene(600.0, 300.0);
    // a bar chart with a part of its bars on a full axis would mislead: none are drawn
    assert_eq!(scene.omitted, 30_000);
    assert!(!scene.items.iter().any(|i| matches!(i, Item::Rect { .. })));
    let note = scene.items.iter().any(
        |i| matches!(i, Item::Text { text, .. } if text == "Marks not drawn, too many: 30000."),
    );
    assert!(note, "the omission is announced");
    let p = scene.plot;
    assert!(
        pic.ink([p[0], p[1] + p[3] + 2.0, p[2], 30.0], pic.bg) > 50,
        "axis still drawn"
    );
}
