//! Calendar ticks for a time axis, in the zone the data was written in.
//!
//! Pattern: Strategy by unit. Ticks sit on the wall clock of a *fixed offset* (the data's own),
//! never on the machine's zone, so they do not move with DST or with the reader's location.
//! The interval is chosen like a person would: the nearest of 1/5/15/30 s, min, 1/3/6/12 h,
//! 1/2 days, week, 1/3 months, then years in steps of 1, 2, 5, 10, ...

use super::ticks::step_for;
use crate::data::calendar::{civil_from_days, days_from_civil, weekday};
use crate::data::time::{DAY, HOUR, MIN};

/// A calendar unit.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    /// Milliseconds (for spans under a second).
    Millisecond,
    /// Seconds.
    Second,
    /// Minutes.
    Minute,
    /// Hours.
    Hour,
    /// Days (day of month).
    Day,
    /// Weeks, starting on Sunday.
    Week,
    /// Months.
    Month,
    /// Years.
    Year,
}

/// `step` units of `unit`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interval {
    /// The unit.
    pub unit: Unit,
    /// How many units between ticks.
    pub step: u32,
}

const CANDIDATES: [(Unit, u32, f64); 18] = [
    (Unit::Second, 1, 1_000.0),
    (Unit::Second, 5, 5_000.0),
    (Unit::Second, 15, 15_000.0),
    (Unit::Second, 30, 30_000.0),
    (Unit::Minute, 1, MIN),
    (Unit::Minute, 5, 5.0 * MIN),
    (Unit::Minute, 15, 15.0 * MIN),
    (Unit::Minute, 30, 30.0 * MIN),
    (Unit::Hour, 1, HOUR),
    (Unit::Hour, 3, 3.0 * HOUR),
    (Unit::Hour, 6, 6.0 * HOUR),
    (Unit::Hour, 12, 12.0 * HOUR),
    (Unit::Day, 1, DAY),
    (Unit::Day, 2, 2.0 * DAY),
    (Unit::Week, 1, 7.0 * DAY),
    (Unit::Month, 1, 30.0 * DAY),
    (Unit::Month, 3, 91.0 * DAY),
    (Unit::Year, 1, 365.0 * DAY),
];

/// The interval for about `count` ticks over `span` milliseconds.
pub fn pick(span: f64, count: usize) -> Interval {
    #[allow(clippy::cast_precision_loss)]
    let target = span / count.max(1) as f64;
    let i = CANDIDATES.partition_point(|c| c.2 <= target);
    if i == CANDIDATES.len() {
        let years = span / (365.0 * DAY);
        let step = step_for(0.0, years.max(1e-9), count).map_or(1.0, |s| s.size().max(1.0));
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        return Interval {
            unit: Unit::Year,
            step: step.min(f64::from(u32::MAX)) as u32,
        };
    }
    if i == 0 {
        let s = step_for(0.0, span.max(1e-9), count).map_or(1.0, |s| s.size().max(1.0));
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        return Interval {
            unit: Unit::Millisecond,
            step: s as u32,
        };
    }
    let (a, b) = (CANDIDATES[i - 1], CANDIDATES[i]);
    let c = if target / a.2 < b.2 / target { a } else { b };
    Interval {
        unit: c.0,
        step: c.1,
    }
}

/// Tick instants (milliseconds on the UTC line) from `t0` to `t1` inclusive, on the wall clock
/// of the fixed offset `offset_min` (minutes east of UTC).
pub fn ticks(t0: f64, t1: f64, iv: Interval, offset_min: i32) -> Vec<f64> {
    let off = f64::from(offset_min) * MIN;
    let (l0, l1) = (t0 + off, t1 + off);
    let mut out: Vec<f64> = Vec::new();
    let fixed = |len: f64, out: &mut Vec<f64>| {
        let mut k = (l0 / len).ceil();
        while k * len <= l1 && out.len() < 10_000 {
            out.push(k * len);
            k += 1.0;
        }
    };
    let step = f64::from(iv.step);
    match iv.unit {
        Unit::Millisecond => fixed(step, &mut out),
        Unit::Second => fixed(step * 1_000.0, &mut out),
        Unit::Minute => fixed(step * MIN, &mut out),
        Unit::Hour => fixed(step * HOUR, &mut out),
        Unit::Day | Unit::Week => {
            #[allow(clippy::cast_possible_truncation)]
            let mut d = (l0 / DAY).ceil() as i64;
            #[allow(clippy::cast_precision_loss)]
            while (d as f64) * DAY <= l1 && out.len() < 10_000 {
                let (_, _, dom) = civil_from_days(d);
                let hit = if iv.unit == Unit::Week {
                    weekday(d) == 0
                } else {
                    (dom - 1) % iv.step == 0
                };
                #[allow(clippy::cast_precision_loss)]
                if hit {
                    out.push(d as f64 * DAY);
                }
                d += 1;
            }
        }
        Unit::Month | Unit::Year => calendar_ticks(l0, l1, iv, &mut out),
    }
    out.into_iter().map(|l| l - off).collect()
}

fn calendar_ticks(l0: f64, l1: f64, iv: Interval, out: &mut Vec<f64>) {
    #[allow(clippy::cast_possible_truncation)]
    let (y0, m0, _) = civil_from_days((l0 / DAY).floor() as i64);
    let day_ms = |y: i64, m: u32| -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let t = days_from_civil(y, m, 1) as f64 * DAY;
        t
    };
    if iv.unit == Unit::Year {
        let step = i64::from(iv.step.max(1));
        let mut y = (y0 + step - 1).div_euclid(step) * step;
        while out.len() < 10_000 && day_ms(y, 1) <= l1 {
            if day_ms(y, 1) >= l0 {
                out.push(day_ms(y, 1));
            }
            y += step;
        }
        return;
    }
    let (mut y, mut m) = (y0, m0);
    while out.len() < 10_000 && day_ms(y, m) <= l1 {
        if (m - 1) % iv.step == 0 && day_ms(y, m) >= l0 {
            out.push(day_ms(y, m));
        }
        if m == 12 {
            (y, m) = (y + 1, 1);
        } else {
            m += 1;
        }
    }
}
