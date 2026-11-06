//! Dashed lines really are dashed on the picture (Blinc 0.6 draws no dashes by itself), and the
//! cutting into dashes is exact.

mod common;

use common::{near, out_dir, render, view};
use fs_components_charts::layout::dash::pieces;
use fs_components_charts::scene::{Dash, Item, Paint};

#[test]
fn dashes_are_cut_at_exact_lengths_and_run_round_corners() {
    let line = [[0.0, 0.0], [100.0, 0.0]];
    let p = pieces(&line, Dash::Dashed);
    assert_eq!(p.len(), 8, "period 13, a dash starts at 0, 13, ... 91");
    assert_eq!(p[0], vec![[0.0, 0.0], [8.0, 0.0]]);
    assert_eq!(p[1], vec![[13.0, 0.0], [21.0, 0.0]]);
    assert_eq!(p[7], vec![[91.0, 0.0], [99.0, 0.0]]);
    // a corner in the middle of a dash: the dash bends
    let bent = pieces(&[[0.0, 0.0], [5.0, 0.0], [5.0, 10.0]], Dash::Dashed);
    assert_eq!(
        bent[0],
        vec![[0.0, 0.0], [5.0, 0.0], [5.0, 3.0]],
        "5 + 3 = 8 long"
    );
    assert_eq!(pieces(&line, Dash::Solid), vec![line.to_vec()]);
    assert_eq!(pieces(&[[1.0, 1.0]], Dash::Dotted).len(), 1);
    assert!(
        pieces(&[[0.0, 0.0], [0.0, 0.0]], Dash::Dashed).is_empty()
            || pieces(&[[0.0, 0.0], [0.0, 0.0]], Dash::Dashed)
                .iter()
                .all(|d| d.len() >= 2)
    );
}

const TWO_LINES: &str = r#"{
  "data": {"values": [
    {"x": 0, "y": 1, "g": "a"}, {"x": 100, "y": 1, "g": "a"},
    {"x": 0, "y": 3, "g": "b"}, {"x": 100, "y": 3, "g": "b"},
    {"x": 0, "y": 5, "g": "c"}, {"x": 100, "y": 5, "g": "c"}]},
  "mark": "line",
  "encoding": {"x": {"field": "x", "type": "quantitative"}, "y": {"field": "y", "type": "quantitative"}, "color": {"field": "g", "type": "nominal"}}
}"#;

/// Number of ink runs along a horizontal pixel row between `x0` and `x1`.
fn runs(pic: &common::Picture, y: f32, x0: f32, x1: f32, colour: [u8; 3]) -> usize {
    let (mut n, mut inside) = (0, false);
    let mut x = x0;
    while x < x1 {
        let hit = (-2..=2).any(|dy| near(pic.pixel(x, y + dy as f32), colour, 110));
        if hit && !inside {
            n += 1;
        }
        inside = hit;
        x += 1.0;
    }
    n
}

#[test]
fn a_dashed_series_has_gaps_on_the_picture_and_a_solid_one_has_none() {
    let (dir, _tmp) = out_dir("dash");
    let v = view(TWO_LINES);
    let pic = render(&v, 520, 320, false, &dir, "dash");
    let scene = v.scene(520.0, 320.0);
    let y_of = |s: usize| -> f32 {
        scene
            .items
            .iter()
            .find_map(|i| match i {
                Item::Path {
                    points,
                    paint: Paint::Series(n),
                    ..
                } if *n == s && points[0][0] < scene.plot[0] + scene.plot[2] => Some(points[0][1]),
                _ => None,
            })
            .expect("a path")
    };
    let p = scene.plot;
    let (x0, x1) = (p[0] + 6.0, p[0] + p[2] - 6.0);
    assert_eq!(
        runs(&pic, y_of(0), x0, x1, pic.series[0]),
        1,
        "series 0 is solid"
    );
    let dashed = runs(&pic, y_of(1), x0, x1, pic.series[1]);
    assert!(
        dashed >= 15,
        "series 1 has {dashed} dashes over {} px",
        x1 - x0
    );
    let dotted = runs(&pic, y_of(2), x0, x1, pic.series[2]);
    assert!(dotted > dashed, "series 2 is dotted: {dotted} runs");
    // the legend shows the same patterns: runs along the sample of each entry
    let sample = |s: usize| {
        scene.items.iter().filter(|i| matches!(i, Item::Path { points, paint: Paint::Series(n), .. } if *n == s && points[0][0] > p[0] + p[2])).count()
    };
    assert_eq!(sample(0), 1);
    assert!(
        sample(1) >= 2 && sample(2) >= 4,
        "{} {}",
        sample(1),
        sample(2)
    );
}
