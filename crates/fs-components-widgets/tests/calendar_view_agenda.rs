//! `calendar-view`: a long agenda costs what a short one costs.
//!
//! Pattern: Input test over the router a window uses. No window is opened. A hang here is a deadlock.

mod common;

use std::sync::Arc;

use blinc_layout::event_router::EventRouter;
use common::{layout, serial};
use fs_components_widgets::{
    CalendarItem, CalendarNames, CalendarPosition, CalendarView, Day, Moment, ViewRange,
};

fn names() -> Arc<CalendarNames> {
    let s = |t: &str| t.to_string();
    Arc::new(CalendarNames::new(
        ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"].map(s),
        [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ]
        .map(s),
        "all day",
        "Nothing here",
    ))
}

fn d(text: &str) -> Day {
    Day::parse(text).unwrap()
}

#[test]
fn a_long_agenda_makes_elements_only_for_what_is_in_view_and_the_wheel_brings_the_rest() {
    let _one = serial();
    let many: Vec<CalendarItem> = (0..5000u16)
        .map(|i| {
            let day = d("2026-11-04").add_days(i64::from(i % 30));
            CalendarItem::timed(
                format!("k{i}"),
                format!("Item {i}"),
                Moment::at(day, 8, i % 60),
                Moment::at(day, 9, 0),
            )
        })
        .collect();
    let position = CalendarPosition::new();
    let v = CalendarView::new(ViewRange::Agenda, d("2026-11-04"), names(), Arc::new(many))
        .named("c")
        .position(&position);
    let started = std::time::Instant::now();
    let mut tree = layout(v.build(900.0, 400.0), 900.0, 400.0);
    eprintln!(
        "agenda of 5000 items built and laid out in {:?}",
        started.elapsed()
    );
    assert!(
        started.elapsed().as_secs() < 5,
        "was 7.5 s with an element per row"
    );
    let shown = |tree: &blinc_layout::prelude::RenderTree, key: &str, day: &str| {
        tree.query_by_id(&format!("c-item-{key}@{day}")).is_some()
    };
    assert!(shown(&tree, "k0", "2026-11-04"), "the first is there");
    assert!(
        !shown(&tree, "k29", "2026-12-03"),
        "the last day is not, yet"
    );
    let mut router = EventRouter::new();
    router.on_mouse_move(&tree, 200.0, 200.0);
    let hit = router
        .hit_test(&tree, 200.0, 200.0)
        .expect("something is there");
    tree.dispatch_scroll_chain(hit.node, &hit.ancestors, 200.0, 200.0, 0.0, -151_000.0);
    tree.tick_scroll_physics(1_000);
    let updates = tree.take_pending_property_updates();
    tree.apply_partial_property_updates(updates);
    tree.process_pending_subtree_rebuilds_routed(None);
    tree.compute_layout(900.0, 400.0);
    assert!(position_moved(&position));
    assert!(
        shown(&tree, "k959", "2026-12-03"),
        "the wheel brought the last line: the latest minute, the last title"
    );
    assert!(!shown(&tree, "k0", "2026-11-04"), "and the first left");
}

fn position_moved(position: &CalendarPosition) -> bool {
    position.list_offset() > 100.0
}

#[test]
fn a_rebuild_with_other_items_of_the_same_number_shows_those() {
    let _one = serial();
    let position = CalendarPosition::new();
    let build = |key: &str| {
        let item = CalendarItem::timed(
            key,
            "Same place",
            Moment::at(d("2026-11-04"), 9, 0),
            Moment::at(d("2026-11-04"), 10, 0),
        );
        let v = CalendarView::new(
            ViewRange::Agenda,
            d("2026-11-04"),
            names(),
            Arc::new(vec![item]),
        )
        .named("c")
        .position(&position);
        layout(v.build(900.0, 400.0), 900.0, 400.0)
    };
    assert!(build("old").query_by_id("c-item-old@2026-11-04").is_some());
    let tree = build("new");
    assert!(tree.query_by_id("c-item-new@2026-11-04").is_some());
    assert!(tree.query_by_id("c-item-old@2026-11-04").is_none());
}
