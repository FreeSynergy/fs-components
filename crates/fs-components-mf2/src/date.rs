//! Neutral display of a date and a time of day (UTC), until the date functions of `mf2` are connected (the same limit as
//! `fs-components-words`: `YYYY-MM-DD`, `HH:MM`).
//!
//! Pattern: none — pure functions.

/// `YYYY-MM-DD` of a Unix second (civil-from-days, proleptic Gregorian).
pub(crate) fn day(unix: i64) -> String {
    let z = unix.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

/// `HH:MM` of a Unix second.
pub(crate) fn clock(unix: i64) -> String {
    let s = unix.rem_euclid(86_400);
    format!("{:02}:{:02}", s / 3600, s % 3600 / 60)
}
