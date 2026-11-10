//! Tests of the view: time limits for many items, scrolling when the zoom changes.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_timeline::*;
use host::{H, W, many, plan_with_today};
use ui::{Input, frame, layout, serial};

/// Seconds for one key press plus frame on a timeline of `n` items at day zoom.
fn press_seconds(n: usize) -> f32 {
    let t = plan_with_today(many(n), host::d(2026, 6, 1), Zoom::Day);
    let mut tree = layout(t.build(W, H), W, H);
    let mut input = Input::new();
    input.focus(&tree, "tl-root");
    let start = std::time::Instant::now();
    input.key(&mut tree, 40);
    frame(&mut tree, W, H);
    start.elapsed().as_secs_f32()
}

#[test]
fn a_hundred_and_twenty_items_redraw_within_the_time_limit() {
    let _g = serial();
    let took = press_seconds(120);
    let limit = if cfg!(debug_assertions) { 30.0 } else { 0.5 };
    assert!(took < limit, "one key press with 120 items took {took} s");
}

/// The measurement series for the Blinc report: `cargo test --release -- --ignored --nocapture series`.
#[test]
#[ignore = "measurement, not a check"]
fn series_of_item_counts() {
    let _g = serial();
    for n in [30, 60, 120, 240, 480] {
        eprintln!("SERIES {n} items: {:.3} s", press_seconds(n));
    }
}
