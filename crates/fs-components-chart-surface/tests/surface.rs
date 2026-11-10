//! The chart screen shows what the source gives, and says why when it cannot.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-charts/tests/common/mod.rs"]
mod common;

use std::sync::{Arc, Mutex};

use common::{out_dir, render};
use fs_components_app::Surface;
use fs_components_chart_surface::{ChartSource, ChartSurface};
use fs_components_charts::Labels;

struct Source(Mutex<String>);
impl ChartSource for Source {
    fn spec(&self) -> String {
        self.0.lock().unwrap().clone()
    }
}

const PER_DAY: &str = r#"{
  "data": {"values": [{"day": "2026-11-09", "program": "calendar"}, {"day": "2026-11-09", "program": "tasks"}, {"day": "2026-11-10", "program": "calendar"}]},
  "mark": "bar",
  "encoding": {"x": {"field": "day", "type": "ordinal"}, "y": {"aggregate": "count", "type": "quantitative"}, "color": {"field": "program", "type": "nominal"}}
}"#;

#[test]
fn a_spec_is_drawn_and_a_new_spec_shows_at_the_next_build() {
    let source = Arc::new(Source(Mutex::new(PER_DAY.to_owned())));
    let surface = ChartSurface::new(source.clone(), Labels::default());
    let (dir, _tmp) = out_dir("surface");
    let view = surface.view().unwrap();
    let pic = render(&view, 480, 320, false, &dir, "per-day");
    assert!(
        pic.ink([0.0, 0.0, 480.0, 320.0], pic.bg) > 2000,
        "something was drawn"
    );
    assert_eq!(view.text_alternative().table.rows.len(), 3);
    // the data changes: the next view has the new rows
    *source.0.lock().unwrap() = PER_DAY.replace("tasks", "calendar");
    assert_eq!(
        surface.view().unwrap().text_alternative().table.rows.len(),
        2
    );
}

#[test]
fn a_spec_outside_the_subset_is_told_not_drawn() {
    let surface = ChartSurface::new(
        Arc::new(Source(Mutex::new("{\"mark\":\"nonsense\"}".into()))),
        Labels::default(),
    );
    assert!(surface.view().is_err());
    // building still gives an element (the notice), never a panic
    let _ = surface.build(300.0, 200.0);
}
