//! Tests of the view: scroll requests: before the build, NaN, the reader scrolling.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_timeline::*;
use host::{H, W, body_width, many, plan_with_today, shown_x, spread};
use ui::{frame, layout, serial};

#[test]
fn a_scroll_asked_for_before_the_first_build_ends_inside_the_content() {
    let _g = serial();
    for n in [0, 1, 60] {
        let t = plan_with_today(spread(n, 365), host::d(2026, 6, 1), Zoom::Day);
        t.set_scroll_x(1.0e9);
        let mut tree = layout(t.build(W, H), W, H);
        frame(&mut tree, W, H);
        frame(&mut tree, W, H);
        let x = shown_x(&mut tree, &t);
        assert!(
            x + W <= body_width(&tree) + 0.5,
            "{n} items: the view at {x} is past the end ({})",
            body_width(&tree)
        );
        assert!(
            (t.scroll_x() - x).abs() < 0.6,
            "{n} items: scroll_x {} and the view {x} agree",
            t.scroll_x()
        );
        assert!(
            (t.max_scroll_x() - (body_width(&tree) - W).max(0.0)).abs() < 0.6,
            "{n} items: the largest scroll is the content less the view"
        );
        if n > 0 {
            assert!(
                (x - t.max_scroll_x()).abs() < 1.0,
                "{n} items: at the end of the content: {x}"
            );
        }
    }
}

#[test]
fn nan_and_infinity_are_no_scroll_position() {
    let _g = serial();
    let t = plan_with_today(many(60), host::d(2026, 6, 1), Zoom::Day);
    let mut tree = layout(t.build(W, H), W, H);
    frame(&mut tree, W, H);
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        t.set_scroll_x(500.0);
        t.set_scroll_x(bad);
        assert_eq!(t.scroll_x(), 0.0, "{bad}");
    }
}

#[test]
fn a_scroll_by_the_reader_after_an_earlier_request_is_not_forgotten_by_the_zoom() {
    let _g = serial();
    let t = plan_with_today(many(60), host::d(2026, 6, 1), Zoom::Day);
    let mut tree = layout(t.build(W, H), W, H);
    frame(&mut tree, W, H);
    t.set_scroll_x(3000.0);
    frame(&mut tree, W, H);
    assert!((shown_x(&mut tree, &t) - 3000.0).abs() < 1.0);
    // later, the reader scrolls back to the start with the wheel
    std::thread::sleep(std::time::Duration::from_millis(150));
    let scroll = tree.query_by_id("tl-scroll").unwrap();
    // (the reader's scroll as the window reports it: the view moves and the ref follows)
    tree.scroll_ref(scroll)
        .expect("bound")
        .set_scroll_offset(0.0, 0.0);
    tree.process_pending_scroll_refs();
    assert_eq!(t.scroll_x(), 0.0, "the view is the one to ask now");
    t.set_zoom(Zoom::Month);
    frame(&mut tree, W, H);
    let x = shown_x(&mut tree, &t);
    assert!(x < 1.0, "the view was at the start, so it stays there: {x}");
}

#[test]
fn a_request_before_the_build_is_bounded_even_when_it_was_read_in_between() {
    let _g = serial();
    for n in [0, 1, 60] {
        let t = plan_with_today(spread(n, 365), host::d(2026, 6, 1), Zoom::Day);
        t.set_scroll_x(1.0e9);
        std::thread::sleep(std::time::Duration::from_millis(150));
        let _ = t.scroll_x(); // reading forgets the request, but Blinc still has it
        let mut tree = layout(t.build(W, H), W, H);
        frame(&mut tree, W, H);
        frame(&mut tree, W, H);
        let x = shown_x(&mut tree, &t);
        assert!(
            x + W <= body_width(&tree) + 0.5,
            "{n} items: the view at {x} is past the end ({})",
            body_width(&tree)
        );
    }
}

/// The reader's wheel along the axis, through the same router and scroll chain a window uses.
fn wheel_along(tree: &mut blinc_layout::prelude::RenderTree, x: f32, y: f32, dx: f32) {
    let mut router = blinc_layout::event_router::EventRouter::new();
    let _ = router.on_mouse_move(tree, x, y);
    if let Some(hit) = router.on_scroll_nested(tree, -dx, 0.0) {
        tree.dispatch_scroll_chain(hit.node, &hit.ancestors, x, y, -dx, 0.0);
    }
    tree.tick_scroll_physics(1_000);
}

#[test]
fn the_wheel_moves_the_view_and_the_handle_knows_it() {
    let _g = serial();
    let t = plan_with_today(many(60), host::d(2026, 6, 1), Zoom::Day);
    let mut tree = layout(t.build(W, H), W, H);
    frame(&mut tree, W, H);
    let scroll = tree.query_by_id("tl-scroll").unwrap();
    let b = tree.get_absolute_bounds(scroll).unwrap();
    wheel_along(&mut tree, b.x + 100.0, b.y + 100.0, 2000.0);
    let moved = shown_x(&mut tree, &t);
    assert!(moved > 500.0, "the wheel moved the view: {moved}");
    assert!(
        (t.scroll_x() - moved).abs() < 1.0,
        "and the handle knows it: {} {moved}",
        t.scroll_x()
    );
}
