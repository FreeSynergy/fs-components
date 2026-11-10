//! Tests of the view: scrolling: zoom keeps the view on content, requests are bounded.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_timeline::*;
use host::{H, W, many, plan_with_today, shown_x, spread};
use ui::{Input, frame, layout, serial};

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

#[test]
fn zooming_by_key_scrolled_left_middle_and_right_keeps_the_view_on_content() {
    let _g = serial();
    for days in [365, 1100] {
        key_zoom_over(days);
    }
}

fn key_zoom_over(days: i64) {
    // (start zoom, key, zoom after)
    let steps = [
        (Zoom::Day, 189, Zoom::Week),
        (Zoom::Week, 189, Zoom::Month),
        (Zoom::Week, 187, Zoom::Day),
        (Zoom::Month, 187, Zoom::Week),
    ];
    for (from, key, to) in steps {
        for fraction in [0.0, 0.5, 1.0] {
            let t = plan_with_today(spread(60, days), host::d(2026, 6, 1), from);
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
