//! Tests of the pure model: days, ranges, spans, lanes, navigation, keys. No window.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

use fs_components_calendar_grid::axis::{day_segments, lanes, minute_at, y_of};
use fs_components_calendar_grid::layout::{shown_days, week_of};
use fs_components_calendar_grid::spans::span_rows;
use fs_components_calendar_grid::*;
use fs_components_widgets::Key;

fn d(y: i32, m: u8, dd: u8) -> Day {
    Day::from_ymd(y, m, dd)
}

#[test]
fn civil_dates_round_trip_and_weekdays_are_right() {
    assert_eq!(d(1970, 1, 1).0, 0);
    assert_eq!(d(2026, 11, 10).ymd(), (2026, 11, 10));
    assert_eq!(d(2026, 11, 10).weekday(), 1, "a Tuesday");
    assert_eq!(d(2024, 2, 29).add_days(1).ymd(), (2024, 3, 1));
    assert_eq!(
        d(2024, 1, 31).add_months(1).ymd(),
        (2024, 2, 29),
        "cut to the month's end"
    );
    assert_eq!(d(2026, 1, 15).add_months(-2).ymd(), (2025, 11, 15));
    assert_eq!(d(2026, 2, 10).days_in_month(), 28);
}

#[test]
fn month_shows_42_days_starting_on_the_week_start() {
    let days = shown_days(Mode::Month, d(2026, 11, 10), 0);
    assert_eq!(days.len(), 42);
    assert_eq!(
        days[0].ymd(),
        (2026, 10, 26),
        "Monday before 1 November (a Sunday)"
    );
    assert_eq!(
        shown_days(Mode::Month, d(2026, 11, 10), 6)[0].ymd(),
        (2026, 11, 1)
    );
    assert_eq!(week_of(d(2026, 11, 8), 0).ymd(), (2026, 11, 2));
    assert_eq!(shown_days(Mode::Day, d(2026, 11, 10), 0).len(), 1);
}

#[test]
fn a_multi_day_item_becomes_cut_bars_in_lanes() {
    let days = shown_days(Mode::Month, d(2026, 11, 10), 0);
    let trip = GridItem::all_day("trip", "Trip", d(2026, 11, 5), d(2026, 11, 12));
    let short = GridItem::timed(
        "m",
        "Meet",
        Moment::new(d(2026, 11, 6), 600),
        Moment::new(d(2026, 11, 6), 660),
    );
    let rows = span_rows(&[trip, short], &days, 7, false);
    let row1 = &rows[1]; // 2-8 November
    assert_eq!(row1.len(), 2);
    assert_eq!(
        (row1[0].first, row1[0].last, row1[0].lane, row1[0].cut_right),
        (3, 6, 0, true)
    );
    assert_eq!(
        (row1[1].first, row1[1].lane),
        (4, 1),
        "the short one goes under the long one"
    );
    let row2 = &rows[2];
    assert_eq!(
        (row2[0].first, row2[0].last, row2[0].cut_left),
        (0, 3, true)
    );
}

#[test]
fn an_item_ending_at_midnight_does_not_touch_the_next_day() {
    let i = GridItem::timed(
        "n",
        "Night",
        Moment::new(d(2026, 11, 5), 1320),
        Moment::new(d(2026, 11, 6), 0),
    );
    assert_eq!(i.last_day(), d(2026, 11, 5));
    assert!(!i.is_span(), "ends at midnight: still one column");
    let long = GridItem::timed(
        "n",
        "Night",
        Moment::new(d(2026, 11, 5), 1320),
        Moment::new(d(2026, 11, 6), 60),
    );
    assert!(long.is_span());
}

#[test]
fn overlapping_items_are_side_by_side_and_short_ones_keep_room() {
    let day = d(2026, 11, 10);
    let items = vec![
        GridItem::timed("a", "A", Moment::new(day, 540), Moment::new(day, 660)),
        GridItem::timed("b", "B", Moment::new(day, 600), Moment::new(day, 720)),
        GridItem::timed("c", "C", Moment::new(day, 900), Moment::new(day, 905)),
    ];
    let placed = lanes(&day_segments(&items, day), 30);
    let by = |i: usize| placed.iter().find(|p| p.segment.index == i).unwrap();
    assert_eq!((by(0).lane, by(0).lanes), (0, 2));
    assert_eq!((by(1).lane, by(1).lanes), (1, 2));
    assert_eq!((by(2).lane, by(2).lanes), (0, 1), "alone in its group");
}

#[test]
fn pixels_and_minutes_agree() {
    assert!((y_of(90, 60.0) - 90.0).abs() < 0.01);
    assert_eq!(minute_at(100.0, 60.0, 30), 90, "snaps down to a slot");
    assert_eq!(minute_at(-5.0, 60.0, 30), 0);
    assert_eq!(minute_at(99999.0, 60.0, 30), 1410);
}

#[test]
fn navigation_messages_step_by_mode_and_today_jumps() {
    let mut s = GridState::new(Mode::Month, d(2026, 11, 10));
    s.today = Some(d(2026, 11, 20));
    s.navigate(Nav::Next);
    assert_eq!(s.anchor.ymd(), (2026, 12, 10));
    s.set_mode(Mode::Week);
    s.navigate(Nav::Previous);
    assert_eq!(s.anchor.ymd(), (2026, 12, 3));
    s.navigate(Nav::Today);
    assert_eq!(s.anchor, d(2026, 11, 20));
    s.set_mode(Mode::Day);
    s.navigate(Nav::Next);
    assert_eq!(s.anchor, d(2026, 11, 21));
}

#[test]
fn keys_move_the_cursor_and_enter_chooses_it() {
    let mut s = GridState::new(Mode::Month, d(2026, 11, 10));
    assert_eq!(s.key(Key::Other(65)), KeyOutcome::Ignored);
    assert_eq!(
        s.key(Key::Activate),
        KeyOutcome::Ignored,
        "nothing selected yet"
    );
    assert_eq!(s.key(Key::Right), KeyOutcome::Moved);
    assert_eq!(s.key(Key::Down), KeyOutcome::Moved);
    let slot = Slot {
        day: d(2026, 11, 18),
        minute: None,
    };
    assert_eq!(s.key(Key::Activate), KeyOutcome::Chosen(slot));
    s.set_mode(Mode::Week);
    s.select_slot(Slot {
        day: d(2026, 11, 10),
        minute: Some(1410),
    });
    s.key(Key::Down);
    assert_eq!(
        s.slot,
        Some(Slot {
            day: d(2026, 11, 11),
            minute: Some(0)
        }),
        "past midnight is the next day"
    );
    assert_eq!(s.anchor, d(2026, 11, 11));
}

#[test]
fn the_vec_source_gives_what_touches_the_range() {
    let src = VecSource(vec![
        GridItem::all_day("in", "In", d(2026, 11, 1), d(2026, 11, 3)),
        GridItem::all_day("out", "Out", d(2026, 12, 1), d(2026, 12, 3)),
    ]);
    let got = src.items(d(2026, 11, 3), d(2026, 11, 9));
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].key, "in");
}
