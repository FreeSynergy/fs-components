//! Writing numbers and times for axis labels with the caller's marks.
//!
//! Pattern: Strategy by value kind. A number gets as many decimals as its tick step needs
//! (step 0.5: one decimal), thousands are grouped; a very large or small number falls back to
//! exponent form instead of printing 300 digits.

use crate::data::calendar::{civil_from_days, weekday};
use crate::data::time::{DAY, HOUR, MIN};
use crate::labels::Labels;

/// A number with `decimals` places, grouped and with the caller's marks.
pub fn number(v: f64, decimals: usize, l: &Labels) -> String {
    if v == 0.0 || (v.abs() < 1e15 && (v.abs() >= 1e-6 || decimals > 6)) {
        let text = format!("{:.*}", decimals, v.abs());
        let (int, frac) = text
            .split_once('.')
            .map_or((text.as_str(), ""), |(a, b)| (a, b));
        let mut grouped = String::new();
        for (i, c) in int.chars().enumerate() {
            if i > 0 && (int.len() - i) % 3 == 0 {
                grouped.push_str(&l.group);
            }
            grouped.push(c);
        }
        if !frac.is_empty() {
            grouped.push_str(&l.decimal);
            grouped.push_str(frac);
        }
        let negative = v < 0.0 && text.chars().any(|c| c.is_ascii_digit() && c != '0');
        return if negative {
            format!("{}{grouped}", l.minus)
        } else {
            grouped
        };
    }
    let text = format!("{:e}", v.abs());
    let text = text.replace('.', &l.decimal);
    if v < 0.0 {
        format!("{}{text}", l.minus)
    } else {
        text
    }
}

/// Shortest text of a number without fixed decimals (for categories that are numbers, tooltips
/// and the data table): integers plain, others with up to 12 significant digits.
pub fn plain(v: f64, l: &Labels) -> String {
    let s = if v == v.trunc() && v.abs() < 1e15 {
        format!("{v:.0}")
    } else if format!("{v}").contains('e') || v.abs() < 1e-6 {
        format!("{v:e}")
    } else {
        format!("{v}")
    };
    let s = s.replace('.', &l.decimal);
    s.replace('-', &l.minus)
}

/// A time label from a pattern, for the wall clock `local_ms` (milliseconds on the zone's clock).
pub fn time(local_ms: f64, pattern: &str, l: &Labels) -> String {
    #[allow(clippy::cast_possible_truncation)]
    let days = (local_ms / DAY).floor() as i64;
    let rest = local_ms - days as f64 * DAY;
    let (y, mo, d) = civil_from_days(days);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (h, mi, s, ms) = (
        (rest / HOUR).floor() as u32,
        ((rest % HOUR) / MIN).floor() as u32,
        ((rest % MIN) / 1000.0).floor() as u32,
        (rest % 1000.0).floor() as u32,
    );
    let mut out = String::new();
    let mut chars = pattern.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let piece = match chars.next() {
            Some('Y') => y.to_string(),
            Some('m') => format!("{mo:02}"),
            Some('d') => format!("{d:02}"),
            Some('H') => format!("{h:02}"),
            Some('I') => format!("{:02}", if h % 12 == 0 { 12 } else { h % 12 }),
            Some('M') => format!("{mi:02}"),
            Some('S') => format!("{s:02}"),
            Some('L') => format!("{ms:03}"),
            Some('p') => (if h < 12 { &l.am } else { &l.pm }).clone(),
            Some('a') => l.weekdays_short[weekday(days) as usize].clone(),
            Some('b') => l.months_short[mo as usize - 1].clone(),
            Some('B') => l.months[mo as usize - 1].clone(),
            Some('%') | None => "%".to_string(),
            Some(other) => format!("%{other}"),
        };
        out.push_str(&piece);
    }
    out
}

/// The label for a tick at `local_ms`: coarser patterns for boundaries, as d3's multi-format.
pub fn tick_label(local_ms: f64, l: &Labels) -> String {
    let p = &l.time;
    let days = (local_ms / DAY).floor();
    let rest = local_ms - days * DAY;
    let (_, mo, d) = civil_from_days(days as i64);
    let pattern = if rest % 1000.0 != 0.0 {
        &p.millisecond
    } else if rest % MIN != 0.0 {
        &p.second
    } else if rest % HOUR != 0.0 {
        &p.minute
    } else if rest != 0.0 {
        &p.hour
    } else if d != 1 {
        if weekday(days as i64) == 0 {
            &p.week
        } else {
            &p.day
        }
    } else if mo != 1 {
        &p.month
    } else {
        &p.year
    };
    time(local_ms, pattern, l)
}

/// An instant as ISO 8601 text in its own zone: `2024-03-10T12:00:00+01:00`, or without the
/// offset for a wall-clock time. Whole minutes lose their seconds, and a wall-clock midnight is
/// written as the date alone.
pub fn iso(ms: f64, zone: crate::data::time::Zone) -> String {
    use crate::data::time::Zone;
    let off = match zone {
        Zone::Naive => 0,
        Zone::Offset(o) => o,
    };
    let local = ms + f64::from(off) * MIN;
    let whole = local.rem_euclid(DAY);
    let pattern = if whole == 0.0 && zone == Zone::Naive {
        "%Y-%m-%d"
    } else if whole % MIN == 0.0 {
        "%Y-%m-%dT%H:%M"
    } else if whole % 1000.0 == 0.0 {
        "%Y-%m-%dT%H:%M:%S"
    } else {
        "%Y-%m-%dT%H:%M:%S.%L"
    };
    let text = time(local, pattern, &Labels::default());
    match zone {
        Zone::Naive => text,
        Zone::Offset(0) => text + "Z",
        Zone::Offset(o) => {
            let sign = if o < 0 { '-' } else { '+' };
            format!("{text}{sign}{:02}:{:02}", o.abs() / 60, o.abs() % 60)
        }
    }
}
