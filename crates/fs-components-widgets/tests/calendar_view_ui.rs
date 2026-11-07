//! `calendar-view`: the real element, clicked over the router and hit test a window uses.
//!
//! Tests: Input tests. No window is opened. A hang here is a deadlock.

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, layout, serial};
use fs_components_widgets::calendar_view::OnPick;
use fs_components_widgets::{
    CalendarItem, CalendarNames, CalendarView, Day, Moment, Picked, ViewRange,
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
fn a_click_on_a_day_reports_the_day_and_an_item_only_the_item() {
    let _one = serial();
    let (v, picks) = view(ViewRange::Month, sample());
    let mut tree = layout(v.build(900.0, 600.0), 900.0, 600.0);
    let mut input = Input::new();
    input.click(&mut tree, "c-day-2026-11-20");
    assert_eq!(*picks.lock().unwrap(), [Picked::Day(d("2026-11-20"))]);
    picks.lock().unwrap().clear();
    input.click(&mut tree, "c-item-a@2026-11-04");
    assert_eq!(
        *picks.lock().unwrap(),
        [Picked::Item("a".into())],
        "the day under the item hears nothing"
    );
}

#[test]
fn on_the_axis_too_an_item_reports_only_itself() {
    let _one = serial();
    let (v, picks) = view(ViewRange::Week, sample());
    let mut tree = layout(v.build(900.0, 2000.0), 900.0, 2000.0);
    let mut input = Input::new();
    input.click(&mut tree, "c-item-b@2026-11-04");
    assert_eq!(*picks.lock().unwrap(), [Picked::Item("b".into())]);
    picks.lock().unwrap().clear();
    input.click(&mut tree, "c-item-h@2026-11-05");
    assert_eq!(*picks.lock().unwrap(), [Picked::Item("h".into())]);
    picks.lock().unwrap().clear();
    let (x, _) = Input::centre(&tree, "c-day-2026-11-07");
    input.click_at(&mut tree, x, 300.0);
    assert_eq!(*picks.lock().unwrap(), [Picked::Day(d("2026-11-07"))]);
}

#[test]
fn an_item_over_midnight_is_on_both_days_and_overlapping_ones_are_side_by_side() {
    let _one = serial();
    let (v, _) = view(ViewRange::Week, sample());
    let tree = layout(v.build(900.0, 2000.0), 900.0, 2000.0);
    assert!(tree.query_by_id("c-item-c@2026-11-05").is_some());
    assert!(tree.query_by_id("c-item-c@2026-11-06").is_some());
    let bounds = |id: &str| {
        tree.get_absolute_bounds(tree.query_by_id(id).unwrap())
            .unwrap()
    };
    let (a, b) = (bounds("c-item-a@2026-11-04"), bounds("c-item-b@2026-11-04"));
    assert!(a.x + a.width <= b.x + 0.5, "{a:?} {b:?}");
    assert!(a.width > 40.0, "still wide enough to read: {a:?}");
}

#[test]
fn more_than_fit_in_a_month_cell_are_counted_and_the_count_reports_the_day() {
    let _one = serial();
    let many: Vec<CalendarItem> = (0..9)
        .map(|i| {
            CalendarItem::timed(
                format!("k{i}"),
                format!("Item {i}"),
                at("2026-11-11", 8 + i, 0),
                at("2026-11-11", 8 + i, 30),
            )
        })
        .collect();
    let (v, picks) = view(ViewRange::Month, Arc::new(many));
    let mut tree = layout(v.build(900.0, 600.0), 900.0, 600.0);
    assert!(tree.query_by_id("c-item-k0@2026-11-11").is_some());
    assert!(tree.query_by_id("c-item-k8@2026-11-11").is_none());
    let mut input = Input::new();
    input.click(&mut tree, "c-more-2026-11-11");
    assert_eq!(*picks.lock().unwrap(), [Picked::More(d("2026-11-11"))]);
}

#[test]
fn a_crowd_on_the_axis_keeps_readable_lanes_and_counts_the_rest() {
    let _one = serial();
    let many: Vec<CalendarItem> = (0..12)
        .map(|i| {
            CalendarItem::timed(
                format!("k{i}"),
                "Same",
                at("2026-11-04", 9, 0),
                at("2026-11-04", 12, 0),
            )
        })
        .collect();
    let (v, picks) = view(ViewRange::Week, Arc::new(many));
    let mut tree = layout(v.build(900.0, 2000.0), 900.0, 2000.0);
    let first = tree
        .get_absolute_bounds(tree.query_by_id("c-item-k0@2026-11-04").unwrap())
        .unwrap();
    assert!(first.width >= 50.0, "{first:?}");
    assert!(tree.query_by_id("c-item-k11@2026-11-04").is_none());
    let mut input = Input::new();
    input.click(&mut tree, "c-more-2026-11-04");
    assert_eq!(*picks.lock().unwrap(), [Picked::More(d("2026-11-04"))]);
}
