//! Reading a whole spec.
//!
//! Pattern: none (pure functions; `parse` is the one entry point over the small readers) — everything it does
//! not understand becomes a named refusal, never an ignored property.

use serde_json::Value;

use super::channel::channel;
use super::check::check;
use super::error::{Result, SpecError};
use super::json::Obj;
use super::mark::mark;
use super::model::{Channel, Encoding, Spec};
use crate::spec::ErrorKind;

/// Read a Vega-Lite spec from JSON text.
///
/// # Errors
/// [`SpecError`] if the text is not JSON, uses anything outside the subset, or is inconsistent.
pub fn parse(text: &str) -> Result<Spec> {
    let value: Value = serde_json::from_str(text)
        .map_err(|e| SpecError::new(ErrorKind::NotJson, "", e.to_string()))?;
    from_value(&value)
}

/// Read a spec that is already a JSON value.
///
/// # Errors
/// As [`parse`], without the JSON error.
pub fn from_value(value: &Value) -> Result<Spec> {
    let mut o = Obj::new(value, "")?;
    let mut major = 6;
    if let Some(s) = o.string("$schema")? {
        major = schema_major(&s).ok_or_else(|| {
            SpecError::outside(
                "/$schema",
                format!("schema \"{s}\" is not a Vega-Lite 5 or 6 schema address"),
            )
        })?;
    }
    let title = title(&mut o)?;
    let description = o.string("description")?;
    let (width, height) = (size(&mut o, "width")?, size(&mut o, "height")?);
    let data = data(&mut o)?;
    let mark_value = o
        .take("mark")
        .ok_or_else(|| SpecError::invalid("", "\"mark\" is missing"))?;
    let mark = mark(mark_value, "/mark")?;
    let encoding = match o.take("encoding") {
        Some(v) => encoding(v, "/encoding")?,
        None => Encoding::default(),
    };
    let _ = o.take("usermeta");
    let _ = o.string("name")?;
    o.finish()?;
    let spec = Spec {
        schema_major: major,
        title,
        description,
        width,
        height,
        mark,
        encoding,
        data,
    };
    check(&spec)?;
    Ok(spec)
}

fn title(o: &mut Obj<'_>) -> Result<Option<String>> {
    match o.take("title") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(v @ Value::Object(_)) => {
            let mut t = Obj::new(v, &o.at("title"))?;
            let text = t.string("text")?;
            t.finish()?;
            Ok(text)
        }
        Some(v) => Err(o.wrong("title", "a string or {\"text\": ...}", v)),
    }
}

fn size(o: &mut Obj<'_>, key: &'static str) -> Result<Option<f64>> {
    match o.take(key) {
        None => Ok(None),
        Some(Value::Number(n)) => match n.as_f64() {
            Some(x) if x.is_finite() && x > 0.0 && x <= 1e6 => Ok(Some(x)),
            _ => Err(SpecError::invalid(
                &o.at(key),
                "must be a number of pixels above 0 and up to 1000000",
            )),
        },
        Some(v) => Err(SpecError::outside(
            &o.at(key),
            format!("{key} {v} is not supported (a number of pixels)"),
        )),
    }
}

fn data(o: &mut Obj<'_>) -> Result<Vec<serde_json::Map<String, Value>>> {
    let v = o
        .take("data")
        .ok_or_else(|| SpecError::invalid("", "\"data\" is missing"))?;
    let path = o.at("data");
    let mut d = Obj::new(v, &path)?;
    let values = d.take("values").ok_or_else(|| {
        SpecError::outside(&path, "only inline data (\"values\") is supported; the chart does not load files or named datasets")
    })?;
    let Value::Array(items) = values else {
        return Err(SpecError::outside(
            &d.at("values"),
            "\"values\" must be an array of objects",
        ));
    };
    let mut rows = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        match item {
            Value::Object(m) => rows.push(m.clone()),
            _ => {
                return Err(SpecError::invalid(
                    &format!("{path}/values/{i}"),
                    "every row must be an object",
                ));
            }
        }
    }
    d.finish()?;
    Ok(rows)
}

fn encoding(v: &Value, path: &str) -> Result<Encoding> {
    let o = Obj::new(v, path)?;
    let mut out = Encoding::default();
    for key in o.keys() {
        let ch = match key {
            "x" => Channel::X,
            "y" => Channel::Y,
            "color" => Channel::Color,
            "theta" => Channel::Theta,
            other => {
                return Err(SpecError::outside(
                    &o.at(other),
                    format!("channel \"{other}\" is not supported (x, y, color, theta)"),
                ));
            }
        };
        let def = channel(o.get(key), ch, &o.at(key))?;
        match ch {
            Channel::X => out.x = Some(def),
            Channel::Y => out.y = Some(def),
            Channel::Color => out.color = Some(def),
            Channel::Theta => out.theta = Some(def),
        }
    }
    Ok(out)
}

/// The major version of an address like `https://vega.github.io/schema/vega-lite/v5.17.0.json`
/// (only 5 and 6, with up to two more numbers, and the `.json`).
fn schema_major(s: &str) -> Option<u8> {
    let rest = s.strip_prefix("https://vega.github.io/schema/vega-lite/v")?;
    let name = rest.strip_suffix(".json")?;
    let mut parts = name.split('.');
    let major: u8 = parts.next()?.parse().ok()?;
    let more = parts.collect::<Vec<_>>();
    let numeric = more.len() <= 2
        && more
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
    (matches!(major, 5 | 6) && numeric).then_some(major)
}
