//! Tests of the view: drawn offscreen and driven by clicks and keys, as a window would.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::{Arc, Mutex};

use fs_components_calendar_grid::*;
use fs_components_fields::{KeysAsText, Look};
use fs_components_styles::StyleId;
use ui::{Input, draw_tree, frame, layout, serial, size_of};

const W: f32 = 900.0;
const H: f32 = 640.0;

#[derive(Default)]
struct Log(Mutex<Vec<String>>);

impl GridEvents for Log {
    fn select_slot(&self, s: Slot) {
        self.0
            .lock()
            .unwrap()
            .push(format!("slot {:?} {:?}", s.day.ymd(), s.minute));
    }
    fn select_item(&self, key: &str) {
        self.0.lock().unwrap().push(format!("item {key}"));
    }
    fn navigated(&self, nav: Nav, a: Day) {
        self.0
            .lock()
            .unwrap()
            .push(format!("nav {nav:?} {:?}", a.ymd()));
    }
}

fn d(y: i32, m: u8, dd: u8) -> Day {
    Day::from_ymd(y, m, dd)
}

fn grid(style: StyleId) -> (CalendarGrid, Arc<Log>) {
    let items = vec![
        GridItem::all_day("trip", "Trip", d(2026, 11, 9), d(2026, 11, 12)),
        GridItem::timed(
            "sync",
            "Sync",
            Moment::new(d(2026, 11, 10), 600),
            Moment::new(d(2026, 11, 10), 660),
        ),
        GridItem::timed(
            "call",
            "Call",
            Moment::new(d(2026, 11, 10), 630),
            Moment::new(d(2026, 11, 10), 690),
        ),
    ];
    let mut st = GridState::new(Mode::Month, d(2026, 11, 10));
    st.today = Some(d(2026, 11, 10));
    st.now = Some(Moment::new(d(2026, 11, 10), 615));
    let log = Arc::new(Log::default());
    let g = CalendarGrid::new(
        st,
        Arc::new(VecSource(items)),
        Arc::new(KeysAsText),
        Look::new(style, false),
    )
    .events(log.clone());
    (g, log)
}

fn last(log: &Log) -> String {
    log.0.lock().unwrap().last().cloned().unwrap_or_default()
}

#[test]
fn the_month_draws_cells_bars_and_the_today_mark_in_every_style() {
    let _g = serial();
    for style in StyleId::ALL {
        let (g, _) = grid(style);
        let tree = layout(g.build(W, H), W, H);
        for i in [0, 20, 41] {
            assert!(
                tree.query_by_id(&format!("cal-cell-{i}")).is_some(),
                "{style:?} cell {i}"
            );
        }
        assert!(
            tree.query_by_id("cal-item-trip-2").is_some(),
            "{style:?}: the bar in its week row"
        );
        let (w, _) = size_of(&tree, "cal-cell-3");
        assert!((w - W / 7.0).abs() < 1.0, "a seventh of the width");
        if ui::has_fonts() {
            let img = draw_tree(&tree, W as u32, H as u32, "calendar-grid-month.png");
            assert!(ui::ink(&img, (0, 0, W as u32, H as u32), ui::background(&img)) > 500);
        }
    }
}

#[test]
fn clicking_a_cell_a_bar_and_the_buttons_tells_the_host_and_changes_the_grid() {
    let _g = serial();
    let (g, log) = grid(StyleId::Admin);
    let mut tree = layout(g.build(W, H), W, H);
    let mut input = Input::new();
    input.click(&mut tree, "cal-cell-10");
    frame(&mut tree, W, H);
    assert_eq!(last(&log), "slot (2026, 11, 5) None");
    assert_eq!(g.snapshot().slot.unwrap().day, d(2026, 11, 5));
    input.click(&mut tree, "cal-item-trip-2");
    frame(&mut tree, W, H);
    assert_eq!(last(&log), "item trip");
    assert!(g.snapshot().slot.is_none(), "one thing is selected");
    input.click(&mut tree, "cal-next");
    frame(&mut tree, W, H);
    assert_eq!(last(&log), "nav Next (2026, 12, 10)");
    assert!(
        tree.query_by_id("cal-item-trip-2").is_none(),
        "the bar is gone with its month"
    );
    input.click(&mut tree, "cal-today");
    frame(&mut tree, W, H);
    assert!(tree.query_by_id("cal-item-trip-2").is_some(), "and back");
}

#[test]
fn week_and_day_have_columns_an_all_day_row_overlapping_items_and_a_now_line() {
    let _g = serial();
    let (g, log) = grid(StyleId::Windows);
    let mut tree = layout(g.build(W, H), W, H);
    let mut input = Input::new();
    input.click(&mut tree, "cal-mode-week");
    frame(&mut tree, W, H);
    for id in [
        "cal-col-0",
        "cal-col-6",
        "cal-allday",
        "cal-item-trip",
        "cal-item-sync",
        "cal-item-call",
        "cal-now",
    ] {
        assert!(tree.query_by_id(id).is_some(), "no {id}");
    }
    let (sw, _) = size_of(&tree, "cal-item-sync");
    let (cw, _) = size_of(&tree, "cal-col-1");
    assert!(
        sw < cw * 0.6,
        "sync and call overlap, side by side ({sw} of {cw})"
    );
    input.click(&mut tree, "cal-allday-5");
    frame(&mut tree, W, H);
    assert_eq!(last(&log), "slot (2026, 11, 14) None");
    // a click in the sixth hour of the Tuesday column
    let node = tree.query_by_id("cal-col-1").unwrap();
    let b = tree.get_absolute_bounds(node).unwrap();
    let hour = b.height / 24.0;
    input.click_at(&mut tree, b.x + 20.0, b.y + hour * 6.0 + 5.0);
    frame(&mut tree, W, H);
    assert_eq!(last(&log), "slot (2026, 11, 10) Some(360)");
    assert!(
        tree.query_by_id("cal-slot").is_some(),
        "the chosen slot is marked"
    );
    input.click(&mut tree, "cal-mode-day");
    frame(&mut tree, W, H);
    assert!(tree.query_by_id("cal-col-0").is_some() && tree.query_by_id("cal-col-1").is_none());
}

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
