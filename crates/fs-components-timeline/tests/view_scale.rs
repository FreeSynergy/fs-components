//! Tests of the view: time limits for many items, scrolling when the zoom changes.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_timeline::*;
use host::{H, W, plan_with_today};
use ui::{Input, frame, layout, serial};

fn many(n: usize) -> Vec<Item> {
    (0..n)
        .map(|i| {
            let start = host::d(2026, 1, 1).add_days((i as i64 * 365) / n as i64);
            Item::bar(&format!("i{i}"), "Task", start, start.add_days(9), 50)
        })
        .collect()
}

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

#[test]
fn zooming_out_while_scrolled_far_right_keeps_the_view_on_content() {
    let _g = serial();
    let t = plan_with_today(many(60), host::d(2026, 6, 1), Zoom::Day);
    let mut tree = layout(t.build(W, H), W, H);
    frame(&mut tree, W, H);
    let scroll = tree.query_by_id("tl-scroll").unwrap();
    t.set_scroll_x(3000.0);
    tree.process_pending_scroll_refs();
    frame(&mut tree, W, H);
    assert!(
        (tree.get_scroll_offset(scroll).0 + 3000.0).abs() < 1.0,
        "scrolled: {:?}",
        tree.get_scroll_offset(scroll)
    );
    t.set_zoom(Zoom::Month);
    frame(&mut tree, W, H);
    tree.process_pending_scroll_refs();
    let x = -tree.get_scroll_offset(scroll).0;
    let max = t.max_scroll_x();
    assert!(
        max < 3000.0,
        "the month axis is shorter than the old offset: {max}"
    );
    assert!(
        x <= max + 0.5,
        "the offset {x} is past the end of the content ({max})"
    );
    assert!(
        (x - 300.0).abs() < 2.0,
        "the same day stays at the left: {x}"
    );
    // and the other way: from the end of the month axis to days, still inside the content
    t.set_scroll_x(t.max_scroll_x());
    tree.process_pending_scroll_refs();
    t.set_zoom(Zoom::Day);
    frame(&mut tree, W, H);
    tree.process_pending_scroll_refs();
    assert!(-tree.get_scroll_offset(scroll).0 <= t.max_scroll_x() + 0.5);
}
