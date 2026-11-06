//! `calendar-view`: dates, spans, parts on a day and lanes. Pure, no window.
//!
//! Pattern: Table tests over the layout functions.

use fs_components_widgets::calendar_view::{
    Segment, lanes, month_days, segment_on, span_of, step_from, week_of,
};
use fs_components_widgets::{CalendarItem, Day, Moment, ViewRange};

fn d(text: &str) -> Day {
    Day::parse(text).unwrap()
}

fn at(day: &str, hour: u16, minute: u16) -> Moment {
    Moment::at(d(day), hour, minute)
}

#[test]
fn days_count_across_centuries_and_leap_years() {
    assert_eq!(d("1970-01-01").number(), 0);
    assert_eq!(d("2000-03-01").number() - d("2000-02-28").number(), 2);
    assert_eq!(d("1900-03-01").number() - d("1900-02-28").number(), 1);
    assert_eq!(d("2026-11-06").weekday(), 4, "a Friday");
    for number in [-1_000_000, -1, 0, 59, 20_000, 2_932_896, 10_000_000] {
        assert_eq!(Day::from_number(number).number(), number);
    }
    assert_eq!(d("2026-12-31").add_days(1), d("2027-01-01"));
    assert_eq!(
        d("3000-02-28").add_days(1),
        d("3000-03-01"),
        "not a leap year"
    );
}

#[test]
fn only_real_dates_are_dates() {
    for bad in [
        "2026-02-29",
        "2026-13-01",
        "2026-00-10",
        "26-1-1",
        "2026-1-1",
        "x",
    ] {
        assert_eq!(Day::parse(bad), None, "{bad}");
    }
    assert!(Day::parse("2028-02-29").is_some());
    assert_eq!(d("2028-02-29").to_string(), "2028-02-29");
}

#[test]
fn a_month_step_from_the_31st_lands_on_the_last_of_a_short_month() {
    assert_eq!(d("2027-01-31").add_months(1), d("2027-02-28"));
    assert_eq!(d("2027-03-31").add_months(-1), d("2027-02-28"));
    assert_eq!(d("2026-12-15").add_months(1), d("2027-01-15"));
    assert_eq!(d("2027-01-15").add_months(-1), d("2026-12-15"));
}

#[test]
fn the_week_begins_where_the_caller_says() {
    let friday = d("2026-11-06");
    assert_eq!(week_of(friday, 0), d("2026-11-02"), "Monday");
    assert_eq!(week_of(friday, 6), d("2026-11-01"), "Sunday");
    assert_eq!(
        week_of(d("2026-11-01"), 6),
        d("2026-11-01"),
        "already its first day"
    );
    assert_eq!(
        week_of(d("2026-11-01"), 0),
        d("2026-10-26"),
        "the Sunday ends a Monday week"
    );
}

#[test]
fn six_weeks_cover_every_month_with_either_week_start() {
    for start in [0, 6] {
        for month in 1..=12 {
            let anchor = Day::new(2026, month, 15).unwrap();
            let days = month_days(anchor, start);
            assert_eq!(days.len(), 42);
            assert_eq!(days[0].weekday(), start);
            assert!(days.contains(&anchor.first_of_month()));
            let last = anchor.first_of_month().add_months(1).add_days(-1);
            assert!(days.contains(&last), "{last}");
        }
    }
    assert_eq!(month_days(d("2026-11-20"), 0)[0], d("2026-10-26"));
}

#[test]
fn spans_and_steps_of_each_view() {
    let a = d("2026-11-06");
    assert_eq!(
        span_of(ViewRange::Month, a, 0, 30),
        (d("2026-10-26"), d("2026-12-06"))
    );
    assert_eq!(
        span_of(ViewRange::Week, a, 6, 30),
        (d("2026-11-01"), d("2026-11-07"))
    );
    assert_eq!(span_of(ViewRange::Day, a, 0, 30), (a, a));
    assert_eq!(span_of(ViewRange::Agenda, a, 0, 30), (a, d("2026-12-05")));
    assert_eq!(step_from(ViewRange::Month, a, true, 30), d("2026-12-06"));
    assert_eq!(step_from(ViewRange::Week, a, false, 30), d("2026-10-30"));
    assert_eq!(step_from(ViewRange::Day, a, true, 30), d("2026-11-07"));
    assert_eq!(step_from(ViewRange::Agenda, a, true, 30), d("2026-12-06"));
}

#[test]
fn an_item_over_midnight_has_a_part_on_each_day() {
    let item = CalendarItem::timed("k", "t", at("2026-11-06", 22, 0), at("2026-11-07", 2, 0));
    assert_eq!(item.start.clock(), "22:00");
    assert_eq!(
        segment_on(&item, d("2026-11-06")),
        Some(Segment {
            start: 1320,
            end: 1440
        })
    );
    assert_eq!(
        segment_on(&item, d("2026-11-07")),
        Some(Segment { start: 0, end: 120 })
    );
    assert_eq!(segment_on(&item, d("2026-11-08")), None);
    assert_eq!(segment_on(&item, d("2026-11-05")), None);
}

#[test]
fn an_item_ending_at_midnight_is_not_on_the_next_day_and_a_point_is_on_its_day() {
    let item = CalendarItem::timed("k", "t", at("2026-11-06", 22, 0), at("2026-11-07", 0, 0));
    assert!(segment_on(&item, d("2026-11-06")).is_some());
    assert_eq!(segment_on(&item, d("2026-11-07")), None);
    let point = CalendarItem::timed("k", "t", at("2026-11-07", 0, 0), at("2026-11-07", 0, 0));
    assert_eq!(
        segment_on(&point, d("2026-11-07")),
        Some(Segment { start: 0, end: 0 })
    );
}

#[test]
fn an_all_day_item_covers_its_days_only_and_crosses_a_month() {
    let item = CalendarItem::all_day("k", "t", d("2026-11-30"), d("2026-12-01"));
    assert!(item.all_day);
    assert_eq!(segment_on(&item, d("2026-11-29")), None);
    assert!(segment_on(&item, d("2026-11-30")).is_some());
    assert!(segment_on(&item, d("2026-12-01")).is_some());
    assert_eq!(segment_on(&item, d("2026-12-02")), None);
}

#[test]
fn an_end_before_the_start_is_not_taken() {
    let item = CalendarItem::timed("k", "t", at("2026-11-06", 10, 0), at("2026-11-06", 9, 0));
    assert_eq!(item.end, item.start);
}

fn seg(start: u16, end: u16) -> Segment {
    Segment { start, end }
}

fn placed(segments: &[Segment], shortest: u16) -> Vec<(usize, usize)> {
    let mut places = lanes(segments, shortest);
    places.sort_by_key(|p| p.index);
    places.iter().map(|p| (p.lane, p.lanes)).collect()
}

#[test]
fn what_overlaps_goes_side_by_side_and_the_rest_keeps_the_whole_width() {
    assert_eq!(placed(&[seg(60, 120), seg(180, 240)], 30), [(0, 1), (0, 1)]);
    assert_eq!(placed(&[seg(60, 180), seg(120, 240)], 30), [(0, 2), (1, 2)]);
    assert_eq!(
        placed(&[seg(60, 300), seg(60, 120), seg(120, 180)], 30),
        [(0, 2), (1, 2), (1, 2)],
        "a lane that is free is used again"
    );
    assert_eq!(
        placed(&[seg(60, 120), seg(120, 180)], 30),
        [(0, 1), (0, 1)],
        "one ending when the other begins does not overlap"
    );
}

#[test]
fn a_short_item_keeps_the_room_it_needs_to_be_read() {
    assert_eq!(
        placed(&[seg(60, 65), seg(70, 90)], 30),
        [(0, 2), (1, 2)],
        "five minutes are drawn as thirty"
    );
    assert_eq!(placed(&[seg(60, 60), seg(60, 60)], 30), [(0, 2), (1, 2)]);
}

#[test]
fn nothing_is_lost_in_a_crowd() {
    let many: Vec<Segment> = (0..50).map(|i| seg(60 + i, 200)).collect();
    let places = lanes(&many, 30);
    assert_eq!(places.len(), 50);
    let mut seen: Vec<usize> = places.iter().map(|p| p.lane).collect();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), 50, "each in a lane of its own");
    assert!(lanes(&[], 30).is_empty());
}
