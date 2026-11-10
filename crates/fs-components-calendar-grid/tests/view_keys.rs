//! Tests of the view: keys, and what the host hears of navigation.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;

use fs_components_calendar_grid::*;
use fs_components_fields::{KeysAsText, Look};
use fs_components_styles::StyleId;
use host::{H, W, d, grid, last};
use ui::{Input, frame, layout, serial};

#[test]
fn the_keyboard_moves_the_cursor_and_enter_chooses() {
    let _g = serial();
    let (g, log) = grid(StyleId::Apple);
    let mut tree = layout(g.build(W, H), W, H);
    let mut input = Input::new();
    input.focus(&tree, "cal-root");
    input.key(&mut tree, 39); // right
    input.key(&mut tree, 40); // down: a week on
    frame(&mut tree, W, H);
    assert_eq!(g.snapshot().slot.unwrap().day, d(2026, 11, 18));
    input.key(&mut tree, 13); // enter
    assert_eq!(last(&log), "slot (2026, 11, 18) None");
    input.key(&mut tree, 34); // page down
    frame(&mut tree, W, H);
    assert_eq!(g.snapshot().anchor.ymd().1, 12);
}

#[test]
fn page_keys_home_and_arrows_over_a_range_edge_tell_the_host() {
    let _g = serial();
    let (g, log) = grid(StyleId::Admin);
    let mut tree = layout(g.build(W, H), W, H);
    let mut input = Input::new();
    input.focus(&tree, "cal-root");
    input.key(&mut tree, 34); // page down
    frame(&mut tree, W, H);
    assert_eq!(last(&log), "nav Next (2026, 12, 10)");
    input.key(&mut tree, 36); // home
    frame(&mut tree, W, H);
    assert_eq!(last(&log), "nav Today (2026, 11, 10)");
    input.key(&mut tree, 33); // page up
    frame(&mut tree, W, H);
    assert_eq!(last(&log), "nav Previous (2026, 10, 10)");
    log.0.lock().unwrap().clear();
    input.click(&mut tree, "cal-mode-week");
    frame(&mut tree, W, H);
    input.focus(&tree, "cal-root");
    input.key(&mut tree, 39); // right inside the week: no new range
    assert!(last(&log).is_empty(), "{}", last(&log));
    for _ in 0..5 {
        input.key(&mut tree, 39); // on to the next week
    }
    frame(&mut tree, W, H);
    assert!(last(&log).starts_with("nav Next"), "{}", last(&log));
}

#[test]
fn the_more_mark_stays_inside_its_week_row() {
    let _g = serial();
    let many: Vec<GridItem> = (0..6)
        .map(|i| GridItem::all_day(&format!("e{i}"), "Event", d(2026, 11, 10), d(2026, 11, 10)))
        .collect();
    let mut st = GridState::new(Mode::Month, d(2026, 11, 10));
    st.today = Some(d(2026, 11, 10));
    let g = CalendarGrid::new(
        st,
        Arc::new(VecSource(many)),
        Arc::new(KeysAsText),
        Look::new(StyleId::Admin, false),
    );
    let tree = layout(g.build(W, H), W, H);
    let bounds = |id: &str| {
        tree.get_absolute_bounds(tree.query_by_id(id).unwrap())
            .unwrap()
    };
    let cell = bounds("cal-cell-15"); // 10 November, third row
    let more = bounds("cal-more-2-1");
    assert!(
        more.y + more.height <= cell.y + cell.height + 0.5,
        "{more:?} in {cell:?}"
    );
}
