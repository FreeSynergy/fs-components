//! `calendar-view`: a narrow view, the scroll position kept, no room, and pictures.
//!
//! Tests: Input tests + pixel tests (the pixel tests pass silently without a graphics adapter, see
//! `common`). No window is opened. A hang here is a deadlock.

mod common;

use std::sync::{Arc, Mutex};

use blinc_layout::event_router::EventRouter;
use common::{draw, layout, serial};
use fs_components_widgets::calendar_view::{NARROW_WIDTH, OnPick};
use fs_components_widgets::{
    CalendarItem, CalendarNames, CalendarPosition, CalendarView, Day, Moment, Picked, ViewRange,
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

fn at(day: &str, hour: u16, minute: u16) -> Moment {
    Moment::at(d(day), hour, minute)
}

fn sample() -> Arc<Vec<CalendarItem>> {
    Arc::new(vec![
        CalendarItem::timed(
            "a",
            "Standup",
            at("2026-11-04", 9, 0),
            at("2026-11-04", 9, 30),
        ),
        CalendarItem::timed(
            "b",
            "Review",
            at("2026-11-04", 9, 15),
            at("2026-11-04", 11, 0),
        )
        .in_category(1),
        CalendarItem::timed(
            "c",
            "Night",
            at("2026-11-05", 22, 0),
            at("2026-11-06", 2, 0),
        ),
        CalendarItem::all_day("h", "Holiday", d("2026-11-05"), d("2026-11-06")).in_category(3),
        CalendarItem::timed(
            "s",
            "Clinic",
            at("2026-11-06", 8, 0),
            at("2026-11-06", 9, 0),
        )
        .marked("Sensitive"),
    ])
}

/// A view with a recorder of what was clicked.
fn view(
    range: ViewRange,
    items: Arc<Vec<CalendarItem>>,
) -> (CalendarView, Arc<Mutex<Vec<Picked>>>) {
    let picks = Arc::new(Mutex::new(Vec::new()));
    let record = Arc::clone(&picks);
    let on_pick: OnPick = Arc::new(move |p| record.lock().unwrap().push(p));
    let view = CalendarView::new(range, d("2026-11-04"), names(), items)
        .named("c")
        .now(at("2026-11-06", 10, 30))
        .on_pick(on_pick);
    (view, picks)
}

#[test]
fn a_narrow_month_is_an_agenda_and_an_agenda_without_items_says_so() {
    let _one = serial();
    let (v, _) = view(ViewRange::Month, sample());
    assert_eq!(v.range_at(NARROW_WIDTH - 1.0), ViewRange::Agenda);
    assert_eq!(v.range_at(NARROW_WIDTH), ViewRange::Month);
    let tree = layout(v.build(300.0, 500.0), 300.0, 500.0);
    assert!(tree.query_by_id("c-item-a@2026-11-04").is_some());
    assert!(
        tree.query_by_id("c-day-2026-11-20").is_none(),
        "no empty days"
    );
    let (empty, _) = view(ViewRange::Agenda, Arc::new(Vec::new()));
    let tree = layout(empty.build(500.0, 300.0), 500.0, 300.0);
    assert!(tree.query_by_id("c-day-2026-11-04").is_none());
}

#[test]
fn the_axis_starts_at_the_hour_asked_once_and_then_stays() {
    let _one = serial();
    let position = CalendarPosition::new();
    let build = |hour: u16| {
        let (v, _) = view(ViewRange::Day, sample());
        let v = v.position(&position).start_hour(hour);
        layout(v.build(500.0, 400.0), 500.0, 400.0)
    };
    let _tree = build(8);
    assert!((position.axis_offset() - 8.0 * 44.0).abs() < 0.5);
    let _tree = build(3);
    assert!(
        (position.axis_offset() - 8.0 * 44.0).abs() < 0.5,
        "the first time only"
    );
}

#[test]
fn no_room_at_all_does_not_crash() {
    let _one = serial();
    for range in [
        ViewRange::Month,
        ViewRange::Week,
        ViewRange::Day,
        ViewRange::Agenda,
    ] {
        let (v, _) = view(range, sample());
        let _tree = layout(v.build(0.0, 0.0), 1.0, 1.0);
    }
}

#[test]
fn the_views_draw_and_differ() {
    let _one = serial();
    let pictures: Vec<_> = [
        (ViewRange::Month, "calendar-view-month.png"),
        (ViewRange::Week, "calendar-view-week.png"),
        (ViewRange::Day, "calendar-view-day.png"),
        (ViewRange::Agenda, "calendar-view-agenda.png"),
    ]
    .into_iter()
    .map(|(range, name)| {
        let (v, _) = view(range, sample());
        draw(v.build(900.0, 600.0), 900, 600, name)
    })
    .collect();
    for (i, a) in pictures.iter().enumerate() {
        for b in &pictures[i + 1..] {
            assert_ne!(a.as_raw(), b.as_raw());
        }
    }
}

#[test]
fn the_wheel_moves_the_axis_and_a_rebuild_keeps_the_place() {
    let _one = serial();
    let position = CalendarPosition::new();
    let build = || {
        let (v, _) = view(ViewRange::Day, sample());
        let v = v.position(&position).start_hour(7);
        layout(v.build(500.0, 400.0), 500.0, 400.0)
    };
    let mut tree = build();
    let at_start = position.axis_offset();
    let mut router = EventRouter::new();
    router.on_mouse_move(&tree, 200.0, 200.0);
    let hit = router
        .hit_test(&tree, 200.0, 200.0)
        .expect("something is there");
    tree.dispatch_scroll_chain(hit.node, &hit.ancestors, 200.0, 200.0, 0.0, -100.0);
    tree.tick_scroll_physics(1_000);
    let moved = position.axis_offset();
    assert!(moved > at_start + 50.0, "{at_start} {moved}");
    let _tree = build();
    assert!(
        (position.axis_offset() - moved).abs() < 0.5,
        "the place stays"
    );
}
