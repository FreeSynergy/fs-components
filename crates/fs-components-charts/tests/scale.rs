//! Scales and ticks give exact numbers, checked by hand.

use fs_components_charts::data::time::{DAY, HOUR};
use fs_components_charts::labels::Labels;
use fs_components_charts::scale::format::{number, tick_label, time};
use fs_components_charts::scale::ticks::{nice, ticks};
use fs_components_charts::scale::timeticks::{Interval, Unit, pick, ticks as time_ticks};
use fs_components_charts::scale::{Band, DomainRule, Linear, domain};

const RULE: DomainRule = DomainRule { zero: true, nice: true, fixed: None };

#[test]
fn ticks_are_round_and_exact_decimals() {
    assert_eq!(ticks(0.0, 100.0, 10), (0..=10).map(|i| f64::from(i) * 10.0).collect::<Vec<_>>());
    assert_eq!(ticks(0.0, 1.0, 10), vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]);
    assert_eq!(ticks(0.0, 10.0, 5), vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
    assert_eq!(ticks(-1.0, 1.0, 4), vec![-1.0, -0.5, 0.0, 0.5, 1.0]);
    assert_eq!(ticks(3.0, 3.0, 5), vec![3.0]);
    assert!(ticks(0.0, f64::INFINITY, 5).is_empty());
    assert!(ticks(5.0, 1.0, 5).is_empty());
}

#[test]
fn nice_is_the_documented_example_of_vega_lite() {
    // the Vega-Lite scale documentation: [0.201479, 0.996679] becomes [0.2, 1.0]
    assert_eq!(nice(0.201_479, 0.996_679, 10), (0.2, 1.0));
    assert_eq!(nice(0.0, 83.7, 10), (0.0, 90.0));
    assert_eq!(nice(0.0, 55.0, 10), (0.0, 55.0));
    assert_eq!(nice(-3.2, 17.0, 10), (-4.0, 18.0));
}

#[test]
fn domain_rules() {
    assert_eq!(domain(2.0, 8.0, RULE), (0.0, 8.0), "zero included, then nice");
    assert_eq!(domain(-5.0, -2.0, RULE), (-5.0, 0.0), "all negative: zero is the top");
    let no_zero = DomainRule { zero: false, ..RULE };
    assert_eq!(domain(1000.0, 1010.0, no_zero), (1000.0, 1010.0));
    let fixed = DomainRule { fixed: Some([0.0, 7.0]), ..RULE };
    assert_eq!(domain(2.0, 100.0, fixed), (0.0, 7.0));
    // all values equal: zero keeps a real domain; without zero it is widened around the value
    assert_eq!(domain(5.0, 5.0, RULE), (0.0, 5.0));
    assert_eq!(domain(5.0, 5.0, no_zero), (2.5, 7.5));
    assert_eq!(domain(0.0, 0.0, RULE), (-1.0, 1.0));
}

#[test]
fn linear_maps_and_survives_huge_domains() {
    let s = Linear::from_data(0.0, 100.0, RULE, (200.0, 0.0));
    assert_eq!((s.d0, s.d1), (0.0, 100.0));
    assert_eq!(s.map(0.0), 200.0);
    assert_eq!(s.map(50.0), 100.0);
    assert_eq!(s.map(100.0), 0.0);
    let huge = Linear::from_data(-1.7e308, 1.7e308, RULE, (0.0, 100.0));
    assert!(huge.d0.is_finite() && huge.d1.is_finite(), "{huge:?}");
    assert!((huge.map(0.0) - 50.0).abs() < 1e-9);
    assert!(huge.map(1.7e308).is_finite());
    assert!(!huge.ticks(5).is_empty());
    let flat = Linear { d0: 3.0, d1: 3.0, r0: 0.0, r1: 10.0 };
    assert_eq!(flat.map(3.0), 5.0, "a one-point domain sits in the middle");
}

#[test]
fn band_and_point_scales_follow_vega() {
    let b = Band::bars(3, (0.0, 300.0));
    assert_eq!((b.start(0), b.bandwidth(), b.step()), (5.0, 90.0, 100.0));
    assert_eq!(b.centre(2), 250.0);
    let p = Band::points(3, (0.0, 300.0));
    assert_eq!((p.centre(0), p.centre(1), p.centre(2), p.bandwidth()), (50.0, 150.0, 250.0, 0.0));
    let one = Band::bars(1, (0.0, 100.0));
    assert!((one.bandwidth() - 90.0).abs() < 1e-9 && (one.start(0) - 5.0).abs() < 1e-9);
    let none = Band::bars(0, (0.0, 100.0));
    assert_eq!(none.n, 0);
}

#[test]
fn numbers_use_the_callers_marks() {
    let l = Labels::default();
    assert_eq!(number(1_234_567.0, 0, &l), "1,234,567");
    assert_eq!(number(-0.5, 1, &l), "\u{2212}0.5");
    assert_eq!(number(-0.0, 1, &l), "0.0");
    assert_eq!(number(-0.04, 1, &l), "0.0", "rounds to zero: no minus");
    assert_eq!(number(1e300, 0, &l), "1e300");
    let mut de = Labels::default();
    (de.decimal, de.group, de.minus) = (",".into(), ".".into(), "-".into());
    assert_eq!(number(-1234.5, 2, &de), "-1.234,50");
}

#[test]
fn time_ticks_sit_on_the_wall_clock_of_the_data() {
    let l = Labels::default();
    let t0 = 1_710_028_800_000.0; // 2024-03-10T00:00Z
    let iv = pick(5.0 * DAY, 5);
    assert_eq!(iv, Interval { unit: Unit::Day, step: 1 });
    let t = time_ticks(t0, t0 + 3.0 * DAY, iv, 0);
    assert_eq!(t, vec![t0, t0 + DAY, t0 + 2.0 * DAY, t0 + 3.0 * DAY]);
    assert_eq!(tick_label(t[0], &l), "Mar 10", "a Sunday gets the week label");
    assert_eq!(tick_label(t[1], &l), "Mon 11", "other days the day label");
    // the same instants on a +02:00 wall clock: midnight there is 22:00Z the day before
    let t = time_ticks(t0, t0 + 2.0 * DAY, iv, 120);
    assert_eq!(t, vec![t0 - 2.0 * HOUR + DAY, t0 - 2.0 * HOUR + 2.0 * DAY]);
}

#[test]
fn time_intervals_and_labels_follow_the_boundaries() {
    let l = Labels::default();
    let jan1 = 1_704_067_200_000.0; // 2024-01-01T00:00Z
    let year = Interval { unit: Unit::Year, step: 1 };
    let t = time_ticks(jan1 - 1.0, jan1 + 800.0 * DAY, year, 0);
    assert_eq!(t.len(), 3, "2024, 2025, 2026");
    assert_eq!(tick_label(t[0], &l), "2024");
    let month = Interval { unit: Unit::Month, step: 3 };
    let t = time_ticks(jan1, jan1 + 366.0 * DAY, month, 0); // 2024 is a leap year
    assert_eq!(t.len(), 5, "Jan, Apr, Jul, Oct, Jan");
    assert_eq!(tick_label(t[1], &l), "April");
    assert_eq!(tick_label(t[4], &l), "2025");
    assert_eq!(time(jan1 + 15.0 * HOUR, "%H:%M %I %p %a %b %d %%", &l), "15:00 03 PM Mon Jan 01 %");
    assert_eq!(tick_label(jan1 + 15.5 * HOUR, &l), "03:30");
    assert_eq!(pick(400.0 * 365.0 * DAY, 10), Interval { unit: Unit::Year, step: 50 });
    let week = Interval { unit: Unit::Week, step: 1 };
    let t = time_ticks(jan1, jan1 + 14.0 * DAY, week, 0);
    assert_eq!(t.len(), 2, "Sundays: Jan 7 and Jan 14");
    assert_eq!(tick_label(t[0], &l), "Jan 07");
}

#[test]
fn a_zero_or_reversed_time_span_gives_no_trouble() {
    let iv = pick(0.0, 5);
    assert_eq!(iv.unit, Unit::Millisecond);
    assert!(time_ticks(10.0, 5.0, Interval { unit: Unit::Day, step: 1 }, 0).is_empty());
}
