//! Field paths: how a name in the spec finds a value in a row.
//!
//! Pattern: none — Vega-Lite reads `a.b` as the nested `a` then `b`, and `a\.b`
//! as one key with a dot. We follow that, and refuse bracket paths.

use serde_json::{Map, Value};

use super::error::{Result, SpecError};

/// Refuse paths we do not read the way Vega-Lite does.
pub(super) fn check_path(field: &str, path: &str) -> Result<()> {
    if field.is_empty() {
        return Err(SpecError::invalid(path, "the field name is empty"));
    }
    if field.contains(['[', ']']) {
        return Err(SpecError::outside(
            path,
            "bracket field paths (a[0]) are not supported",
        ));
    }
    Ok(())
}

/// The segments of a field path.
pub fn segments(field: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut chars = field.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'.') => {
                chars.next();
                push(&mut out, '.');
            }
            '.' => out.push(String::new()),
            c => push(&mut out, c),
        }
    }
    out
}

fn push(out: &mut [String], c: char) {
    if let Some(last) = out.last_mut() {
        last.push(c);
    }
}

/// The value of `field` in `row`, or `None` if any step is missing.
pub fn lookup<'a>(row: &'a Map<String, Value>, field: &str) -> Option<&'a Value> {
    let segs = segments(field);
    let (first, rest) = segs.split_first()?;
    let mut cur = row.get(first)?;
    for s in rest {
        cur = cur.as_object()?.get(s)?;
    }
    Some(cur)
}
