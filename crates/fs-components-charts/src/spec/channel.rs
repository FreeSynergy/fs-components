//! Reading one channel definition (`encoding/x`, `encoding/color`, ...).
//!
//! Pattern: none of the table fits (small reader functions, no builder type) — each property has its own small reader; the combination rules live
//! in `check.rs`, so a property is understood in one place and judged in another.

use serde_json::Value;

use super::error::{Result, SpecError};
use super::json::{Obj, kind_of};
use super::model::{
    Aggregate, AxisDef, Channel, ChannelDef, FieldType, ScaleDef, Sort, StackMode, Title,
};

/// Read the definition of `channel`.
pub(super) fn channel(value: &Value, channel: Channel, path: &str) -> Result<ChannelDef> {
    let mut o = Obj::new(value, path)?;
    let field = o.string("field")?;
    if let Some(f) = &field {
        super::field::check_path(f, &o.at("field"))?;
    }
    let aggregate = aggregate(&mut o)?;
    let given = o.string("type")?;
    if let Some(t) = &given {
        field_type(t, &o.at("type"))?; // a type that is no type is refused whatever else is said
    }
    let ftype = match (given, aggregate) {
        // a count is a number whatever type is written (Vega-Lite uses quantitative, with a warning)
        (_, Some(Aggregate::Count)) => FieldType::Quantitative,
        (Some(t), _) => field_type(&t, &o.at("type"))?,
        (None, _) => {
            return Err(SpecError::invalid(
                path,
                "\"type\" is missing (quantitative, nominal, ordinal or temporal)",
            ));
        }
    };
    if field.is_none() && aggregate != Some(Aggregate::Count) {
        return Err(SpecError::invalid(path, "\"field\" is missing"));
    }
    if aggregate.is_some_and(|a| a != Aggregate::Count) && ftype != FieldType::Quantitative {
        return Err(SpecError::outside(
            &o.at("aggregate"),
            "an aggregate other than count needs a quantitative field",
        ));
    }
    let title = title(o.take("title"), &o)?;
    let (axis, scale, sort, stack, legend) = (
        axis(&mut o, channel)?,
        scale(&mut o, channel, ftype)?,
        sort(&mut o, channel, ftype)?,
        stack(&mut o, channel)?,
        legend(&mut o, channel)?,
    );
    let mut axis = axis;
    if axis.title == Title::Auto {
        axis.title = title.clone();
    }
    o.finish()?;
    Ok(ChannelDef {
        field,
        ftype,
        aggregate,
        title,
        axis,
        scale,
        sort,
        stack,
        legend,
    })
}

fn field_type(t: &str, path: &str) -> Result<FieldType> {
    match t {
        "quantitative" => Ok(FieldType::Quantitative),
        "nominal" => Ok(FieldType::Nominal),
        "ordinal" => Ok(FieldType::Ordinal),
        "temporal" => Ok(FieldType::Temporal),
        other => Err(SpecError::outside(
            path,
            format!("type \"{other}\" is not supported (quantitative, nominal, ordinal, temporal)"),
        )),
    }
}

fn aggregate(o: &mut Obj<'_>) -> Result<Option<Aggregate>> {
    let Some(name) = o.string("aggregate")? else {
        return Ok(None);
    };
    match name.as_str() {
        "count" => Ok(Some(Aggregate::Count)),
        "sum" => Ok(Some(Aggregate::Sum)),
        "mean" | "average" => Ok(Some(Aggregate::Mean)),
        "min" => Ok(Some(Aggregate::Min)),
        "max" => Ok(Some(Aggregate::Max)),
        other => Err(SpecError::outside(
            &o.at("aggregate"),
            format!("aggregate \"{other}\" is not supported (count, sum, mean, min, max)"),
        )),
    }
}

fn title(v: Option<&Value>, o: &Obj<'_>) -> Result<Title> {
    match v {
        None => Ok(Title::Auto),
        Some(Value::Null) => Ok(Title::Hidden),
        Some(Value::String(s)) => Ok(Title::Text(s.clone())),
        Some(v) => Err(o.wrong("title", "a string or null", v)),
    }
}

fn axis(o: &mut Obj<'_>, ch: Channel) -> Result<AxisDef> {
    let Some(v) = o.take("axis") else {
        return Ok(AxisDef::default());
    };
    if !matches!(ch, Channel::X | Channel::Y) {
        return Err(SpecError::invalid(
            &o.at("axis"),
            "only x and y have an axis",
        ));
    }
    if v.is_null() {
        return Ok(AxisDef {
            visible: false,
            ..AxisDef::default()
        });
    }
    let mut a = Obj::new(v, &o.at("axis"))?;
    let title = title(a.take("title"), &a)?;
    let out = AxisDef {
        visible: true,
        title,
        grid: a.boolean("grid")?,
        labels: a.boolean("labels")?.unwrap_or(true),
    };
    a.finish()?;
    Ok(out)
}

fn scale(o: &mut Obj<'_>, ch: Channel, ft: FieldType) -> Result<ScaleDef> {
    let Some(v) = o.take("scale") else {
        return Ok(ScaleDef::default());
    };
    let path = o.at("scale");
    if !matches!(ch, Channel::X | Channel::Y) {
        return Err(SpecError::invalid(&path, "only x and y have a scale"));
    }
    let mut s = Obj::new(v, &path)?;
    if let Some(t) = s.string("type")?
        && (t != "linear" || ft != FieldType::Quantitative)
    {
        return Err(SpecError::outside(
            &s.at("type"),
            format!(
                "scale type \"{t}\" is not supported (only the default scale of the field type)"
            ),
        ));
    }
    let domain = match s.take("domain") {
        None => None,
        Some(Value::Array(a)) if a.len() == 2 && ft == FieldType::Quantitative => {
            match (a[0].as_f64(), a[1].as_f64()) {
                (Some(lo), Some(hi)) if lo < hi => Some([lo, hi]),
                _ => {
                    return Err(SpecError::invalid(
                        &s.at("domain"),
                        "needs two numbers, smaller first",
                    ));
                }
            }
        }
        Some(v) => {
            return Err(SpecError::outside(
                &s.at("domain"),
                format!(
                    "domain {} is not supported (only [min, max] of a quantitative field)",
                    kind_of(v)
                ),
            ));
        }
    };
    let out = ScaleDef {
        zero: s.boolean("zero")?,
        nice: s.boolean("nice")?,
        domain,
    };
    if out.nice == Some(true) && ft == FieldType::Temporal {
        return Err(SpecError::outside(
            &s.at("nice"),
            "nice on a time scale rounds to local calendar days in Vega-Lite, which depends on the viewer's zone; not supported",
        ));
    }
    s.finish()?;
    Ok(out)
}

fn sort(o: &mut Obj<'_>, ch: Channel, ft: FieldType) -> Result<Sort> {
    let Some(v) = o.take("sort") else {
        return Ok(Sort::default());
    };
    let path = o.at("sort");
    if !ft.is_discrete() {
        return Err(SpecError::invalid(
            &path,
            "sort applies to nominal and ordinal fields only",
        ));
    }
    let by = |name: &str| match name {
        "x" | "-x" => Some((Channel::X, name.starts_with('-'))),
        "y" | "-y" => Some((Channel::Y, name.starts_with('-'))),
        _ => None,
    };
    match v {
        Value::Null => Ok(Sort::DataOrder),
        Value::Array(a) => Ok(Sort::Values(a.clone())),
        Value::String(s) if s == "ascending" => Ok(Sort::Ascending),
        Value::String(s) if s == "descending" => Ok(Sort::Descending),
        Value::String(s) => match by(s) {
            Some((channel, descending)) if channel != ch && ch != Channel::Color => {
                Ok(Sort::ByChannel {
                    descending,
                    channel,
                })
            }
            _ => Err(SpecError::outside(
                &path,
                format!(
                    "sort \"{s}\" is not supported (ascending, descending, null, a list, or the other axis as x, y, -x, -y)"
                ),
            )),
        },
        other => Err(SpecError::outside(
            &path,
            format!("sort given as {} is not supported", kind_of(other)),
        )),
    }
}

fn stack(o: &mut Obj<'_>, ch: Channel) -> Result<Option<StackMode>> {
    let Some(v) = o.take("stack") else {
        return Ok(None);
    };
    let path = o.at("stack");
    if !matches!(ch, Channel::X | Channel::Y) {
        return Err(SpecError::outside(
            &path,
            "stack on this channel is not supported (only on the quantitative x or y of a bar or area)",
        ));
    }
    match v {
        Value::Null | Value::Bool(false) => Ok(Some(StackMode::Off)),
        Value::Bool(true) => Ok(Some(StackMode::Zero)),
        Value::String(s) if s == "zero" => Ok(Some(StackMode::Zero)),
        Value::String(s) if s == "normalize" => Ok(Some(StackMode::Normalize)),
        other => Err(SpecError::outside(
            &path,
            format!("stack {other} is not supported (zero, normalize, null)"),
        )),
    }
}

fn legend(o: &mut Obj<'_>, ch: Channel) -> Result<bool> {
    let Some(v) = o.take("legend") else {
        return Ok(true);
    };
    if ch != Channel::Color {
        return Err(SpecError::invalid(
            &o.at("legend"),
            "only color has a legend",
        ));
    }
    if v.is_null() {
        return Ok(false);
    }
    let mut l = Obj::new(v, &o.at("legend"))?;
    let t = title(l.take("title"), &l)?;
    l.finish()?;
    match t {
        Title::Auto => Ok(true),
        _ => Err(SpecError::outside(
            &o.at("legend/title"),
            "set the legend title on the channel's \"title\" instead",
        )),
    }
}
