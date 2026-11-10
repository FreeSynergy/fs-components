//! Tests of the view: drawn offscreen and driven by clicks and keys, as a window would.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_styles::StyleId;
use fs_components_timeline::*;
use host::{H, W, last, timeline};
use ui::{Input, background, draw_tree, frame, ink, layout, serial, size_of};

#[test]
fn bars_milestone_progress_and_today_line_are_drawn_in_every_style() {
    let _g = serial();
    for style in StyleId::ALL {
        let (t, _) = timeline(style, true);
        let tree = layout(t.build(W, H), W, H);
        for id in [
            "tl-item-design",
            "tl-item-ship",
            "tl-done-build",
            "tl-today",
            "tl-arrows",
            "tl-zoom-week",
        ] {
            assert!(tree.query_by_id(id).is_some(), "{style:?}: no {id}");
        }
        let (bar, _) = size_of(&tree, "tl-item-design");
        let (done, _) = size_of(&tree, "tl-done-build");
        let (build, _) = size_of(&tree, "tl-item-build");
        assert!(
            (bar - 12.0 * 12.0).abs() < 1.0,
            "12 days at 12 px (week zoom): {bar}"
        );
        assert!((done - build * 0.4).abs() < 1.0, "40 percent of the bar");
    }
}

#[test]
fn arrows_are_drawn_on_the_canvas() {
    let _g = serial();
    let (with, _) = timeline(StyleId::Admin, true);
    let (without, _) = timeline(StyleId::Admin, false);
    let mut counts = Vec::new();
    for t in [with, without] {
        let tree = layout(t.build(W, H), W, H);
        let b = tree
            .get_absolute_bounds(tree.query_by_id("tl-arrows").unwrap())
            .unwrap();
        let img = draw_tree(&tree, W as u32, H as u32, "timeline-arrows.png");
        counts.push(ink(
            &img,
            (b.x as u32, b.y as u32, b.width as u32, b.height as u32),
            background(&img),
        ));
    }
    assert!(counts[0] > counts[1] + 20, "the arrows add ink: {counts:?}");
}

#[test]
fn zoom_buttons_rescale_the_bars_and_tell_the_host() {
    let _g = serial();
    let (t, log) = timeline(StyleId::Windows, true);
    let mut tree = layout(t.build(W, H), W, H);
    let mut input = Input::new();
    input.click(&mut tree, "tl-zoom-day");
    frame(&mut tree, W, H);
    assert_eq!(last(&log), "zoom day");
    let (day_w, _) = size_of(&tree, "tl-item-design");
    input.click(&mut tree, "tl-zoom-month");
    frame(&mut tree, W, H);
    let (month_w, _) = size_of(&tree, "tl-item-design");
    assert!(
        (day_w - 12.0 * 40.0).abs() < 1.0 && (month_w - 12.0 * 4.0).abs() < 1.0,
        "{day_w} {month_w}"
    );
}

#[test]
fn clicking_an_item_and_using_the_keyboard_choose_items() {
    let _g = serial();
    let (t, log) = timeline(StyleId::Apple, true);
    let mut tree = layout(t.build(W, H), W, H);
    let mut input = Input::new();
    input.click(&mut tree, "tl-item-build");
    frame(&mut tree, W, H);
    assert_eq!(last(&log), "item build");
    assert_eq!(t.snapshot().selected.as_deref(), Some("build"));
    input.focus(&tree, "tl-root");
    input.key(&mut tree, 40);
    frame(&mut tree, W, H);
    assert_eq!(last(&log), "item test");
    input.key(&mut tree, 189); // minus: coarser
    frame(&mut tree, W, H);
    assert_eq!(t.snapshot().zoom, Zoom::Month);
    assert_eq!(last(&log), "zoom month");
}

fn plan_with_today(items: Vec<Item>, today: Day, zoom: Zoom) -> Timeline {
    let mut st = TimelineState::new(Some(today));
    st.zoom = zoom;
    Timeline::new(
        st,
        std::sync::Arc::new(VecSource {
            items,
            dependencies: vec![],
        }),
        std::sync::Arc::new(fs_components_fields::KeysAsText),
        fs_components_fields::Look::new(StyleId::Admin, false),
    )
}

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
