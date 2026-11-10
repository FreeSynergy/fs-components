//! Test support: a grid over a few items, and a log of what the host heard.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic, dead_code)]

use std::sync::{Arc, Mutex};

use fs_components_calendar_grid::*;
use fs_components_fields::{KeysAsText, Look};
use fs_components_styles::StyleId;

pub const W: f32 = 900.0;
pub const H: f32 = 640.0;

#[derive(Default)]
pub struct Log(pub Mutex<Vec<String>>);

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

pub fn d(y: i32, m: u8, dd: u8) -> Day {
    Day::from_ymd(y, m, dd)
}

pub fn grid(style: StyleId) -> (CalendarGrid, Arc<Log>) {
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

pub fn last(log: &Log) -> String {
    log.0.lock().unwrap().last().cloned().unwrap_or_default()
}
