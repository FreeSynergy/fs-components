//! Neutral display of a [`Value`] when a template inserts it with `{name}`.
//!
//! Pattern: none — pure functions. Numbers are shown plainly, a date as `YYYY-MM-DD` and a time
//! as `HH:MM` (UTC). Language-specific formatting is not built (see `DESIGN.md`, Unsupported).

use crate::Value;

/// The neutral text of a value.
pub(crate) fn show(value: &Value) -> String {
    match value {
        Value::Text(s) => s.clone(),
        Value::Count(n) => n.to_string(),
        Value::Number(x) => x.to_string(),
        Value::Date(t) => {
            let (y, m, d) = civil(t.div_euclid(86_400));
            format!("{y:04}-{m:02}-{d:02}")
        }
        Value::Time(t) => {
            let s = t.rem_euclid(86_400);
            format!("{:02}:{:02}", s / 3600, s % 3600 / 60)
        }
    }
}

/// Year, month, day of a day count since 1970-01-01 (proleptic Gregorian).
fn civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}
