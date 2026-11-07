//! Which marks accept which channels: the combination rules of the subset.
//!
//! Pattern: Strategy (a list of rules, each a small function) — a spec that passes each
//! rule means what Vega-Lite says it means; one that does not is refused with the rule's name.

use super::error::{Result, SpecError};
use super::model::{ChannelDef, FieldType, MarkKind, Spec, StackMode};

/// Check the whole spec.
pub(super) fn check(spec: &Spec) -> Result<()> {
    let e = &spec.encoding;
    if let Some(c) = &e.color {
        if c.ftype != FieldType::Nominal {
            return Err(SpecError::outside(
                "/encoding/color/type",
                format!(
                    "colour by a {:?} field is a sequential ramp; only nominal colour is supported",
                    c.ftype
                ),
            ));
        }
        if c.aggregate.is_some() {
            return Err(SpecError::invalid(
                "/encoding/color/aggregate",
                "a colour field is not aggregated",
            ));
        }
    }
    let kind = spec.mark.kind;
    match spec.mark.kind {
        MarkKind::Arc => {
            let t = need(e.theta.as_ref(), "theta", kind)?;
            if t.ftype != FieldType::Quantitative {
                return Err(SpecError::invalid(
                    "/encoding/theta/type",
                    "theta must be quantitative",
                ));
            }
            if e.x.is_some() || e.y.is_some() {
                return Err(SpecError::invalid(
                    "/encoding",
                    "an arc uses theta and color, not x or y",
                ));
            }
        }
        kind => {
            if e.theta.is_some() {
                return Err(SpecError::invalid(
                    "/encoding/theta",
                    "theta belongs to arc marks",
                ));
            }
            let (x, y) = (
                need(e.x.as_ref(), "x", kind)?,
                need(e.y.as_ref(), "y", kind)?,
            );
            position(kind, x, y)?;
        }
    }
    Ok(())
}

fn position(kind: MarkKind, x: &ChannelDef, y: &ChannelDef) -> Result<()> {
    use FieldType::{Quantitative, Temporal};
    match kind {
        MarkKind::Bar => {
            let (disc, quant, qname) = if x.ftype.is_discrete() {
                (x, y, "y")
            } else {
                (y, x, "x")
            };
            if !disc.ftype.is_discrete() || quant.ftype != Quantitative {
                return Err(SpecError::outside(
                    "/encoding",
                    "a bar needs one nominal or ordinal axis and one quantitative axis (bars over time or two number axes are not supported)",
                ));
            }
            if quant.scale.zero == Some(false) {
                return Err(SpecError::outside(
                    &format!("/encoding/{qname}/scale/zero"),
                    "bars without a zero baseline are not supported",
                ));
            }
        }
        MarkKind::Line | MarkKind::Area => {
            if y.ftype != Quantitative {
                return Err(SpecError::outside(
                    "/encoding/y/type",
                    "line and area need a quantitative y (swapped axes are not supported)",
                ));
            }
            if x.ftype == Quantitative && x.aggregate.is_some() {
                return Err(SpecError::outside(
                    "/encoding/x/aggregate",
                    "an aggregated x on a line is not supported",
                ));
            }
        }
        _ => {}
    }
    for (name, c) in [("x", x), ("y", y)] {
        // an explicit stack means what Vega-Lite says only on the value axis of a bar or an area
        let value_axis = match kind {
            MarkKind::Bar => c.ftype == Quantitative,
            MarkKind::Area => name == "y",
            _ => false,
        };
        if let Some(mode) = c.stack
            && !value_axis
            && !(mode == StackMode::Off && matches!(kind, MarkKind::Line | MarkKind::Point))
        {
            return Err(SpecError::outside(
                &format!("/encoding/{name}/stack"),
                format!(
                    "stack on {} of a {} is not supported (only the quantitative axis of a bar, the y of an area; stack: null is fine on a line or point)",
                    name,
                    kind.name()
                ),
            ));
        }
        if c.scale.zero == Some(true) && c.ftype == Temporal {
            return Err(SpecError::invalid(
                &format!("/encoding/{name}/scale/zero"),
                "a time scale has no zero",
            ));
        }
    }
    Ok(())
}

fn need<'a>(c: Option<&'a ChannelDef>, name: &str, kind: MarkKind) -> Result<&'a ChannelDef> {
    c.ok_or_else(|| {
        SpecError::invalid(
            &format!("/encoding/{name}"),
            format!("a {} mark needs the \"{name}\" channel", kind.name()),
        )
    })
}
