//! Test support: a small plan with bars, a milestone and dependencies, and a log of events.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic, dead_code)]

use std::sync::{Arc, Mutex};

use fs_components_fields::{KeysAsText, Look};
use fs_components_styles::StyleId;
use fs_components_timeline::*;

pub const W: f32 = 900.0;
pub const H: f32 = 500.0;

pub fn d(y: i32, m: u8, dd: u8) -> Day {
    Day::from_ymd(y, m, dd)
}

pub fn items() -> Vec<Item> {
    vec![
        Item::bar("design", "Design", d(2026, 11, 2), d(2026, 11, 13), 100),
        Item::bar("build", "Build", d(2026, 11, 16), d(2026, 12, 4), 40).tone(Tone::Success),
        Item::bar("test", "Test", d(2026, 11, 30), d(2026, 12, 11), 0),
        Item::milestone("ship", "Ship", d(2026, 12, 14)),
    ]
}

pub fn deps() -> Vec<Dependency> {
    let dep = |a: &str, b: &str| Dependency {
        from: a.into(),
        to: b.into(),
    };
    vec![
        dep("design", "build"),
        dep("build", "test"),
        dep("test", "ship"),
    ]
}

#[derive(Default)]
pub struct Log(pub Mutex<Vec<String>>);

impl TimelineEvents for Log {
    fn select_item(&self, key: &str) {
        self.0.lock().unwrap().push(format!("item {key}"));
    }
    fn zoomed(&self, z: Zoom) {
        self.0.lock().unwrap().push(format!("zoom {}", z.name()));
    }
}

pub fn timeline(style: StyleId, with_deps: bool) -> (Timeline, Arc<Log>) {
    let src = VecSource {
        items: items(),
        dependencies: if with_deps { deps() } else { vec![] },
    };
    let log = Arc::new(Log::default());
    let t = Timeline::new(
        TimelineState::new(Some(d(2026, 11, 20))),
        Arc::new(src),
        Arc::new(KeysAsText),
        Look::new(style, false),
    )
    .events(log.clone());
    (t, log)
}

pub fn last(log: &Log) -> String {
    log.0.lock().unwrap().last().cloned().unwrap_or_default()
}

pub fn plan_with_today(items: Vec<Item>, today: Day, zoom: Zoom) -> Timeline {
    plan_in(StyleId::Admin, items, today, zoom)
}

/// The same in any style.
pub fn plan_in(style: StyleId, items: Vec<Item>, today: Day, zoom: Zoom) -> Timeline {
    let mut st = TimelineState::new(Some(today));
    st.zoom = zoom;
    Timeline::new(
        st,
        std::sync::Arc::new(VecSource {
            items,
            dependencies: vec![],
        }),
        std::sync::Arc::new(fs_components_fields::KeysAsText),
        fs_components_fields::Look::new(style, false),
    )
}

pub fn many(n: usize) -> Vec<Item> {
    spread(n, 365)
}

/// `n` bars of ten days spread over `days` days.
pub fn spread(n: usize, days: i64) -> Vec<Item> {
    (0..n)
        .map(|i| {
            let start = d(2026, 1, 1).add_days((i as i64 * days) / n as i64);
            Item::bar(&format!("i{i}"), "Task", start, start.add_days(9), 50)
        })
        .collect()
}

/// The scroll offset the view has now (positive), after the view has done what was asked of it.
pub fn shown_x(tree: &mut blinc_layout::prelude::RenderTree, t: &Timeline) -> f32 {
    tree.process_pending_scroll_refs();
    let scroll = tree.query_by_id("tl-scroll").unwrap();
    let _ = t;
    -tree.get_scroll_offset(scroll).0
}

pub fn body_width(tree: &blinc_layout::prelude::RenderTree) -> f32 {
    tree.get_absolute_bounds(tree.query_by_id("tl-body").unwrap())
        .unwrap()
        .width
}
