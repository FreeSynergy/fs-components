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

/// The scroll offset the view has now (positive), after the view has done what was asked of it.
fn shown_x(tree: &mut blinc_layout::prelude::RenderTree, t: &Timeline) -> f32 {
    tree.process_pending_scroll_refs();
    let scroll = tree.query_by_id("tl-scroll").unwrap();
    let _ = t;
    -tree.get_scroll_offset(scroll).0
}

#[test]
fn zooming_by_key_scrolled_left_middle_and_right_keeps_the_view_on_content() {
    let _g = serial();
    // (start zoom, key, zoom after)
    let steps = [
        (Zoom::Day, 189, Zoom::Week),
        (Zoom::Week, 189, Zoom::Month),
        (Zoom::Week, 187, Zoom::Day),
        (Zoom::Month, 187, Zoom::Week),
    ];
    for (from, key, to) in steps {
        for fraction in [0.0, 0.5, 1.0] {
            let t = plan_with_today(many(60), host::d(2026, 6, 1), from);
            let mut tree = layout(t.build(W, H), W, H);
            let mut input = Input::new();
            frame(&mut tree, W, H);
            let before = fraction * t.max_scroll_x();
            t.set_scroll_x(before);
            frame(&mut tree, W, H);
            assert!(
                (shown_x(&mut tree, &t) - before).abs() < 1.0,
                "{from:?} {fraction}: scrolled"
            );
            input.focus(&tree, "tl-root");
            input.key(&mut tree, key);
            frame(&mut tree, W, H);
            assert_eq!(t.snapshot().zoom, to);
            let (x, max) = (shown_x(&mut tree, &t), t.max_scroll_x());
            let want = (before * to.px_per_day() / from.px_per_day()).min(max);
            assert!(
                x <= max + 0.5,
                "{from:?} to {to:?} at {fraction}: offset {x} past the content {max}"
            );
            assert!(
                (x - want).abs() < 2.0,
                "{from:?} to {to:?} at {fraction}: {x}, wanted {want}"
            );
        }
    }
}

#[test]
fn set_scroll_x_stays_inside_the_content() {
    let _g = serial();
    let t = plan_with_today(many(60), host::d(2026, 6, 1), Zoom::Day);
    let mut tree = layout(t.build(W, H), W, H);
    frame(&mut tree, W, H);
    t.set_scroll_x(1.0e6);
    assert!(
        (t.scroll_x() - t.max_scroll_x()).abs() < 0.5,
        "asked for far too much: {}",
        t.scroll_x()
    );
    frame(&mut tree, W, H);
    let x = shown_x(&mut tree, &t);
    assert!(x <= t.max_scroll_x() + 0.5 && x > 0.0, "{x}");
    t.set_scroll_x(-50.0);
    assert_eq!(t.scroll_x(), 0.0);
}

#[test]
fn two_zoom_changes_without_a_frame_between_use_the_scroll_asked_for() {
    let _g = serial();
    let t = plan_with_today(many(60), host::d(2026, 6, 1), Zoom::Day);
    let mut tree = layout(t.build(W, H), W, H);
    frame(&mut tree, W, H);
    t.set_scroll_x(3000.0);
    assert!(
        (t.scroll_x() - 3000.0).abs() < 0.5,
        "counts at once: {}",
        t.scroll_x()
    );
    t.set_zoom(Zoom::Week);
    t.set_zoom(Zoom::Month);
    assert!(
        (t.scroll_x() - 300.0).abs() < 1.0,
        "3000 px at 40 px a day is 300 px at 4: {}",
        t.scroll_x()
    );
    frame(&mut tree, W, H);
    assert!((shown_x(&mut tree, &t) - 300.0).abs() < 2.0);
}

#[test]
fn before_the_first_build_the_largest_scroll_is_the_axis_width() {
    let t = plan_with_today(many(60), host::d(2026, 6, 1), Zoom::Day);
    assert!(
        t.max_scroll_x() > 1000.0,
        "the width of the view counts as zero: {}",
        t.max_scroll_x()
    );
}
