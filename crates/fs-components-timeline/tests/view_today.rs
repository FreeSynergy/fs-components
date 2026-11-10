//! Tests of the view: today as a line or an edge mark, the time limit for ten years, wide titles.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_timeline::*;
use host::{H, W, plan_with_today};
use ui::{Input, background, draw_tree, frame, ink, layout, serial, size_of};

#[test]
fn today_ten_years_after_the_items_costs_no_time_and_shows_an_edge_mark() {
    let _g = serial();
    let old = vec![Item::bar(
        "old",
        "Old",
        host::d(2016, 3, 1),
        host::d(2016, 3, 20),
        100,
    )];
    let t = plan_with_today(old.clone(), host::d(2026, 11, 20), Zoom::Day);
    let start = std::time::Instant::now();
    let mut tree = layout(t.build(W, H), W, H);
    let mut input = Input::new();
    input.focus(&tree, "tl-root");
    input.key(&mut tree, 40);
    frame(&mut tree, W, H);
    assert!(
        start.elapsed().as_secs() < 5,
        "{:?}: the axis must not stretch to today",
        start.elapsed()
    );
    assert!(
        tree.query_by_id("tl-today-edge").is_some(),
        "the edge mark is there"
    );
    assert!(tree.query_by_id("tl-today").is_none(), "and no line");
    let b = tree
        .get_absolute_bounds(tree.query_by_id("tl-body").unwrap())
        .unwrap();
    assert!(b.width < 3000.0, "the axis is about the items: {}", b.width);
    // before the items: the mark is at the left
    let early = plan_with_today(old, host::d(2010, 1, 1), Zoom::Week);
    let tree = layout(early.build(W, H), W, H);
    assert!(tree.query_by_id("tl-today-edge").is_some());
}

#[test]
fn the_today_line_is_drawn_when_today_is_on_the_axis() {
    let _g = serial();
    let t = plan_with_today(host::items(), host::d(2026, 11, 20), Zoom::Week);
    let tree = layout(t.build(W, H), W, H);
    assert!(tree.query_by_id("tl-today").is_some() && tree.query_by_id("tl-today-edge").is_none());
}

#[test]
fn a_milestone_title_takes_the_room_its_glyphs_need() {
    let _g = serial();
    if !ui::has_fonts() {
        return;
    }
    let wide = "WWWWWWWWWWWWWWWWWWWW";
    let narrow = "iiiiiiiiiiiiiiiiiiii";
    let width_of = |title: &str| {
        let items = vec![Item::milestone("m", title, host::d(2026, 11, 20))];
        let tree = layout(
            plan_with_today(items, host::d(2026, 11, 20), Zoom::Day).build(W, H),
            W,
            H,
        );
        let w = size_of(&tree, "tl-title-m").0;
        if title == "WWWWWWWWWWWWWWWWWWWW" {
            draw_tree(&tree, W as u32, H as u32, "timeline-wide-milestone.png");
        }
        w
    };
    let (w, n) = (width_of(wide), width_of(narrow));
    assert!(
        w > 20.0 * 12.0 * 0.6,
        "twenty W are wider than the old 0.6 estimate: {w}"
    );
    assert!(
        n < w / 2.0,
        "and twenty i are much narrower: {n} against {w}"
    );
}

/// Ink in the left and right half of the element `id`, drawn.
fn halves(tree: &blinc_layout::prelude::RenderTree, id: &str) -> (usize, usize) {
    let b = tree
        .get_absolute_bounds(tree.query_by_id(id).unwrap())
        .unwrap();
    let img = draw_tree(tree, W as u32, H as u32, "timeline-edge.png");
    let bg = background(&img);
    let (x, y, w, h) = (b.x as u32, b.y as u32, b.width as u32, b.height as u32);
    (
        ink(&img, (x, y, w / 2, h), bg),
        ink(&img, (x + w / 2, y, w - w / 2, h), bg),
    )
}

#[test]
fn the_edge_mark_points_towards_today_and_sits_at_the_visible_rim() {
    let _g = serial();
    let old = vec![Item::bar(
        "old",
        "Old",
        host::d(2016, 3, 1),
        host::d(2016, 3, 20),
        100,
    )];
    let rect = |tree: &blinc_layout::prelude::RenderTree, id: &str| {
        tree.get_absolute_bounds(tree.query_by_id(id).unwrap())
            .unwrap()
    };
    // today after the items: a triangle pointing right, at the right rim
    let tree = layout(
        plan_with_today(old.clone(), host::d(2026, 11, 20), Zoom::Day).build(W, H),
        W,
        H,
    );
    let m = rect(&tree, "tl-today-edge");
    assert!(
        m.x + m.width > W - 40.0 && m.x + m.width <= W + 0.5,
        "inside the view at the right rim: {m:?}"
    );
    let (left, right) = halves(&tree, "tl-today-arrow");
    assert!(
        left > right + 5,
        "a right-pointing triangle is fuller on the left: {left} {right}"
    );
    // today before the items: pointing left, at the left rim
    let tree = layout(
        plan_with_today(old, host::d(2010, 1, 1), Zoom::Day).build(W, H),
        W,
        H,
    );
    let m = rect(&tree, "tl-today-edge");
    assert!(m.x >= 0.0 && m.x < 40.0, "at the left rim: {m:?}");
    let (left, right) = halves(&tree, "tl-today-arrow");
    assert!(
        right > left + 5,
        "a left-pointing triangle is fuller on the right: {left} {right}"
    );
}

#[test]
fn ten_years_of_items_at_day_zoom_redraw_within_the_time_limit() {
    let _g = serial();
    let items = vec![
        Item::bar("a", "First", host::d(2016, 1, 1), host::d(2016, 6, 1), 100),
        Item::bar("b", "Last", host::d(2025, 6, 1), host::d(2026, 1, 1), 10),
    ];
    let t = plan_with_today(items, host::d(2026, 3, 1), Zoom::Day);
    let mut tree = layout(t.build(W, H), W, H);
    let mut input = Input::new();
    input.focus(&tree, "tl-root");
    let start = std::time::Instant::now();
    input.key(&mut tree, 40);
    frame(&mut tree, W, H);
    let took = start.elapsed().as_secs_f32();
    // release: well under a tenth of a second; debug builds get room
    let limit = if cfg!(debug_assertions) { 10.0 } else { 0.1 };
    assert!(took < limit, "one key press took {took} s");
    assert!(tree.query_by_id("tl-axis").is_some() && tree.query_by_id("tl-item-b").is_some());
}
