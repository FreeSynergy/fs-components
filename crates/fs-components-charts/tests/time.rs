//! Dates are read as written; no machine zone, no daylight saving surprise.

use fs_components_charts::data::calendar::{civil_from_days, days_from_civil, weekday};
use fs_components_charts::data::time::{Zone, parse};

fn ms(t: &str) -> f64 {
    parse(t).unwrap_or_else(|| panic!("{t} should parse")).ms
}

#[test]
fn known_instants() {
    assert_eq!(ms("1970-01-01T00:00:00Z"), 0.0);
    assert_eq!(ms("2024-03-10T12:00:00Z"), 1_710_072_000_000.0);
    assert_eq!(ms("2024-03-10T12:00:00.250Z"), 1_710_072_000_250.0);
    assert_eq!(ms("1969-12-31T23:59:59Z"), -1000.0);
    assert_eq!(ms("2000-02-29"), 951_782_400_000.0);
}

#[test]
fn the_offset_in_the_text_decides_and_a_text_without_one_stays_wall_clock() {
    assert_eq!(ms("2024-03-10T12:00+01:00"), ms("2024-03-10T11:00Z"));
    assert_eq!(ms("2024-03-10T12:00-0530"), ms("2024-03-10T17:30Z"));
    assert_eq!(ms("2024-03-10 12:00:00+01"), ms("2024-03-10T11:00Z"));
    assert_eq!(parse("2024-03-10T12:00").unwrap().zone, Zone::Naive);
    assert_eq!(parse("2024-03-10T12:00Z").unwrap().zone, Zone::Offset(0));
    assert_eq!(
        parse("2024-03-10T12:00+05:30").unwrap().zone,
        Zone::Offset(330)
    );
    // a naive time is the same number whatever zone the machine is in
    assert_eq!(ms("2024-03-10T12:00"), ms("2024-03-10T12:00Z"));
}

#[test]
fn across_a_daylight_saving_change_the_distance_is_real() {
    // Berlin: +01:00 until 2024-03-31 02:00, then +02:00. Noon to noon is 23 hours.
    let gap = ms("2024-03-31T12:00+02:00") - ms("2024-03-30T12:00+01:00");
    assert_eq!(gap, 23.0 * 3_600_000.0);
    // and the other way, in October: 25 hours
    let gap = ms("2024-10-27T12:00+01:00") - ms("2024-10-26T12:00+02:00");
    assert_eq!(gap, 25.0 * 3_600_000.0);
    // the same wall clock without offsets is always 24 hours: no zone is assumed
    assert_eq!(
        ms("2024-03-31T12:00") - ms("2024-03-30T12:00"),
        24.0 * 3_600_000.0
    );
}

#[test]
fn impossible_and_odd_dates_are_refused() {
    for bad in [
        "",
        "x",
        "2024-02-30",
        "2023-02-29",
        "2024-13-01",
        "2024-00-10",
        "2024-03-10T24:00",
        "2024-03-10T12:60",
        "2024-03-10T12:00:60",
        "2024-03-10T12",
        "2024-3-10",
        "10.03.2024",
        "2024-03-10T12:00+25:00",
        "2024-03-10T12:00 UTC",
        "２０２４-03-10",
        "2024-03-10T12:00Zjunk",
    ] {
        assert!(parse(bad).is_none(), "{bad:?} must not parse");
    }
    assert!(parse("2024-02-29").is_some());
    assert!(parse("2024").is_some() && parse("2024-03").is_some());
}

#[test]
fn calendar_round_trip_and_weekday() {
    for d in [-800_000_i64, -1, 0, 59, 60, 11_016, 19_792, 2_932_896] {
        let (y, m, dd) = civil_from_days(d);
        assert_eq!(days_from_civil(y, m, dd), d, "{d}");
    }
    assert_eq!(civil_from_days(19_792), (2024, 3, 10));
    assert_eq!(weekday(0), 4); // 1970-01-01 was a Thursday
    assert_eq!(weekday(19_792), 0); // 2024-03-10 was a Sunday
}
