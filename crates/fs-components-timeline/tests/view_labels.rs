//! Tests of the view: labels of the axis.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_styles::StyleId;
use fs_components_timeline::*;
use host::{H, W, plan_with_today};
use ui::{background, draw_tree, ink, layout, serial};

#[test]
fn a_month_label_is_cut_at_its_tick_and_a_stub_is_not_drawn() {
    let _g = serial();
    if !ui::has_fonts() {
        return;
    }
    // Week zoom, 12 px a day. The axis starts a week before the item: 24 November, so the November
    // tick is 7 days = 84 px wide and the December label starts at 88.
    let items = vec![Item::bar(
        "a",
        "Winter",
        host::d(2025, 12, 1),
        host::d(2026, 1, 20),
        10,
    )];
    let tree = layout(
        plan_with_today(items, host::d(2026, 6, 1), Zoom::Week).build(W, H),
        W,
        H,
    );
    let img = draw_tree(&tree, W as u32, H as u32, "timeline-month-labels.png");
    let bg = background(&img);
    let b = tree
        .get_absolute_bounds(tree.query_by_id("tl-axis").unwrap())
        .unwrap();
    let (x0, y0) = (b.x as u32, b.y as u32);
    let strip = |x: u32, w: u32| ink(&img, (x0 + x, y0, w, 30), bg);
    assert!(
        strip(4, 76) > 20,
        "the November label is drawn up to its tick"
    );
    assert_eq!(
        strip(84, 4),
        0,
        "and not a pixel beyond it, before the December label"
    );
    // Month zoom, 4 px a day: the axis starts on 28 November, so the November tick is 3 days = 12 px
    // wide. A month name has no room there and is left out (no stub that reads as a word); the
    // December label starts after it and is drawn.
    let items = vec![Item::bar(
        "a",
        "Winter",
        host::d(2025, 12, 5),
        host::d(2026, 3, 20),
        10,
    )];
    let tree = layout(
        plan_with_today(items, host::d(2026, 6, 1), Zoom::Month).build(W, H),
        W,
        H,
    );
    let img = draw_tree(&tree, W as u32, H as u32, "timeline-month-zoom.png");
    let b = tree
        .get_absolute_bounds(tree.query_by_id("tl-axis").unwrap())
        .unwrap();
    let strip = |x: u32, w: u32| ink(&img, (b.x as u32 + x, b.y as u32, w, 30), background(&img));
    assert_eq!(
        strip(2, 9),
        0,
        "nothing in the narrow November tick (its line is at 0 and 12)"
    );
    assert!(strip(16, 120) > 20, "the December label is drawn");
}

#[test]
fn day_numbers_are_drawn_at_day_zoom_in_every_style() {
    let _g = serial();
    if !ui::has_fonts() {
        return;
    }
    for style in StyleId::ALL {
        let items = vec![Item::bar(
            "a",
            "Task",
            host::d(2026, 3, 10),
            host::d(2026, 3, 20),
            10,
        )];
        let tree = layout(
            host::plan_in(style, items, host::d(2026, 6, 1), Zoom::Day).build(W, H),
            W,
            H,
        );
        let img = draw_tree(&tree, W as u32, H as u32, "timeline-day-numbers.png");
        let b = tree
            .get_absolute_bounds(tree.query_by_id("tl-axis").unwrap())
            .unwrap();
        let rh = fs_components_styles::style(style).heights.row_dense;
        // the lower row: ten days, each a tile of 40 px with its number at the left
        for day in 0..10u32 {
            let (x, y) = (b.x as u32 + day * 40 + 3, (b.y + rh) as u32);
            let n = ink(&img, (x, y, 30, rh as u32 - 4), background(&img));
            assert!(n > 8, "{style:?}: no day number in tile {day} ({n} pixels)");
        }
    }
}
