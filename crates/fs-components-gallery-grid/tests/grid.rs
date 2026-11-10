//! The gallery grid: the date groups, the order, the tiles, and the drawn element.
//!
//! Tests: Model tests + one offscreen picture. No window is opened.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_gallery_grid::{GalleryGrid, Grouping, Picture, group_label, sections};

fn pictures() -> Vec<Picture> {
    vec![
        Picture::new("a", "a.png", "image/png").taken("2026-11-03T08:00:00Z"),
        Picture::new("b", "b.jpg", "image/jpeg").taken("2026-11-03T09:00:00Z"),
        Picture::new("c", "c.svg", "image/svg+xml").taken("2026-10-30"),
        Picture::new("d", "d.png", "image/png"),
    ]
}

#[test]
fn a_time_names_its_group() {
    assert_eq!(
        group_label("2026-11-03T08:00:00Z", Grouping::Day),
        "2026-11-03"
    );
    assert_eq!(
        group_label("2026-11-03T08:00:00Z", Grouping::Month),
        "2026-11"
    );
    assert_eq!(group_label("2026-11-03", Grouping::Year), "2026");
    assert_eq!(group_label("", Grouping::Day), "");
    assert_eq!(group_label("yesterday", Grouping::Day), "");
    assert_eq!(group_label("2026-11", Grouping::Day), "");
}

#[test]
fn groups_are_newest_first_and_the_undated_come_last() {
    let by_day = sections(pictures(), Grouping::Day);
    let labels: Vec<_> = by_day.iter().map(|s| s.label.as_str()).collect();
    assert_eq!(labels, ["2026-11-03", "2026-10-30", ""]);
    let keys: Vec<_> = by_day[0].pictures.iter().map(|p| p.key.as_str()).collect();
    assert_eq!(keys, ["b", "a"]);
    let by_month = sections(pictures(), Grouping::Month);
    assert_eq!(by_month.len(), 3);
    assert_eq!(sections(pictures(), Grouping::Year).len(), 2);
}

#[test]
fn the_badge_is_the_short_type_word() {
    assert_eq!(Picture::new("x", "x", "image/svg+xml").badge(), "SVG");
    assert_eq!(Picture::new("x", "x", "image/png").badge(), "PNG");
    assert_eq!(Picture::new("x", "x", "").badge(), "");
}

#[test]
fn the_grid_shows_the_tiles_in_group_order_and_is_drawn() {
    let _g = ui::serial();
    let grid = GalleryGrid::new(pictures(), Grouping::Day, "Nothing here");
    assert_eq!(grid.keys(), ["b", "a", "c", "d"]);
    let tree = ui::layout(grid.build(700.0, 400.0), 700.0, 400.0);
    assert!(tree.query_by_id("fs-gallery-grid").is_some());
    let _ = ui::draw_tree(&tree, 700, 400, "gallery-grid");
}
