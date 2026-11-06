//! Reading `mark`.
//!
//! Pattern: Registry of mark kinds — the names the subset knows, and a named refusal for the
//! Vega-Lite marks it does not.

use serde_json::Value;

use super::error::{Result, SpecError};
use super::json::Obj;
use super::model::{MarkDef, MarkKind};

const OTHER_MARKS: [&str; 14] = [
    "rect",
    "rule",
    "text",
    "tick",
    "circle",
    "square",
    "trail",
    "geoshape",
    "boxplot",
    "errorbar",
    "errorband",
    "image",
    "arc3d",
    "area3d",
];

/// Read a mark given as a string or as an object with `type`.
pub(super) fn mark(value: &Value, path: &str) -> Result<MarkDef> {
    let mut def = MarkDef {
        kind: MarkKind::Point,
        inner_radius: 0.0,
        point: false,
    };
    match value {
        Value::String(name) => def.kind = kind(name, path)?,
        Value::Object(_) => {
            let mut o = Obj::new(value, path)?;
            let name = o
                .string("type")?
                .ok_or_else(|| SpecError::invalid(path, "the mark needs a \"type\""))?;
            def.kind = kind(&name, &o.at("type"))?;
            if let Some(r) = o.number("innerRadius")? {
                if def.kind != MarkKind::Arc || r < 0.0 {
                    return Err(SpecError::invalid(
                        &o.at("innerRadius"),
                        "innerRadius is for arc marks and not negative",
                    ));
                }
                def.inner_radius = r;
            }
            if let Some(p) = o.boolean("point")? {
                if !def.kind.is_path() {
                    return Err(SpecError::invalid(
                        &o.at("point"),
                        "point overlays belong to line and area",
                    ));
                }
                def.point = p;
            }
            if let Some(i) = o.string("interpolate")?
                && (i != "linear" || !def.kind.is_path())
            {
                return Err(SpecError::outside(
                    &o.at("interpolate"),
                    format!("interpolate \"{i}\" is not supported (only linear, on line and area)"),
                ));
            }
            o.finish()?;
        }
        other => {
            return Err(SpecError::invalid(
                path,
                format!("the mark must be a string or an object, found {other}"),
            ));
        }
    }
    Ok(def)
}

fn kind(name: &str, path: &str) -> Result<MarkKind> {
    match name {
        "bar" => Ok(MarkKind::Bar),
        "line" => Ok(MarkKind::Line),
        "area" => Ok(MarkKind::Area),
        "point" => Ok(MarkKind::Point),
        "arc" => Ok(MarkKind::Arc),
        n if OTHER_MARKS.contains(&n) => Err(SpecError::outside(
            path,
            format!("mark \"{n}\" is not supported (bar, line, area, point, arc)"),
        )),
        n => Err(SpecError::invalid(
            path,
            format!("\"{n}\" is not a Vega-Lite mark"),
        )),
    }
}
