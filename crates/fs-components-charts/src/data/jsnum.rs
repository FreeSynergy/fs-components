//! Reading a JSON value as a number the way a JavaScript `Number(value)` does.
//!
//! Pattern: none (pure functions; the JSON kind is chosen by `match`). Vega-Lite takes the values of a number field as they are and
//! lets JavaScript turn them into numbers, so `true` is 1, `[5]` is 5, `"0x10"` is 16 and `""` is
//! 0 (checked against Vega-Lite 6.5.0, see `tests/golden`); `null`, `"abc"`, `[1, 2]` and objects
//! are not numbers and the row is invalid. Only finite results count.

use serde_json::Value;

/// The number `value` stands for, if it stands for a finite one.
pub fn number(value: &Value) -> Option<f64> {
    let x = match value {
        Value::Number(n) => n.as_f64()?,
        Value::Bool(b) => f64::from(u8::from(*b)),
        Value::String(s) => text(s)?,
        Value::Array(a) => match a.as_slice() {
            [] => 0.0,
            [one] => return number(one),
            _ => return None,
        },
        _ => return None,
    };
    x.is_finite().then_some(x)
}

/// A text as a number: surrounding blanks are ignored, an empty text is 0, `0x`, `0o` and `0b`
/// give integers, anything else must be a decimal number (`Infinity` and `NaN` are not finite).
pub fn text(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        return Some(0.0);
    }
    for (prefix, radix) in [
        ("0x", 16),
        ("0X", 16),
        ("0o", 8),
        ("0O", 8),
        ("0b", 2),
        ("0B", 2),
    ] {
        if let Some(digits) = t.strip_prefix(prefix) {
            return u64::from_str_radix(digits, radix).ok().map(|n| n as f64);
        }
    }
    let plain = t.strip_prefix('+').unwrap_or(t);
    let ok = plain
        .bytes()
        .all(|b| b.is_ascii_digit() || matches!(b, b'.' | b'e' | b'E' | b'+' | b'-'));
    if !ok {
        return None; // rejects "inf", "nan", "infinity" that Rust would read
    }
    plain.parse::<f64>().ok()
}
