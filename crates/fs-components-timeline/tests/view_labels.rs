//! Tests of the view: labels of the axis.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

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
    // a tick of two days has room for no word: nothing is drawn in it
    let items = vec![Item::bar(
        "a",
        "Winter",
        host::d(2025, 12, 1),
        host::d(2026, 1, 20),
        10,
    )];
    let tree = layout(
        plan_with_today(items, host::d(2026, 6, 1), Zoom::Month).build(W, H),
        W,
        H,
    );
    let b = tree
        .get_absolute_bounds(tree.query_by_id("tl-axis").unwrap())
        .unwrap();
    let img = draw_tree(&tree, W as u32, H as u32, "timeline-month-zoom.png");
    assert!(
        ink(&img, (b.x as u32, b.y as u32, 60, 30), background(&img)) < 400,
        "the axis draws"
    );
}
