//! A bar chart is drawn offscreen, and the pixels are where the layout says they are.

mod common;

use common::{near, out_dir, render, view};

const BARS: &str = r#"{
  "title": "Sales by region",
  "data": {"values": [{"r": "North", "v": 28}, {"r": "South", "v": 55}, {"r": "East", "v": -10}, {"r": "West", "v": 43}]},
  "mark": "bar",
  "encoding": {"x": {"field": "r", "type": "nominal"}, "y": {"field": "v", "type": "quantitative"}}
}"#;

#[test]
fn bars_are_drawn_at_the_layout_positions_in_the_series_colour() {
    let (dir, _tmp) = out_dir("bars");
    let v = view(BARS);
    let pic = render(&v, 480, 320, false, &dir, "bars");
    let scene = v.scene(480.0, 320.0);
    let colour = pic.series[0];
    let bg = pic.bg;
    let mut bars = 0;
    for item in &scene.items {
        if let fs_components_charts::scene::Item::Rect { x, y, w, h, .. } = item {
            bars += 1;
            assert!(
                near(pic.pixel(x + w / 2.0, y + h / 2.0), colour, 6),
                "bar at {x},{y} is not in the series colour: {:?} vs {colour:?}",
                pic.pixel(x + w / 2.0, y + h / 2.0)
            );
            // the gap just left of the bar shows the surface
            assert!(
                near(pic.pixel(x - 3.0, y + h / 2.0), bg, 40)
                    || pic.pixel(x - 3.0, y + h / 2.0) != colour
            );
        }
    }
    assert_eq!(bars, 4);
    assert!(
        pic.ink([0.0, 0.0, 480.0, 320.0], bg) > 3000,
        "something was drawn"
    );
}
