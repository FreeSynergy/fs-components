//! A datum as words: the same text for a tooltip and for the data table of the text alternative.
//!
//! Pattern: Adapter from `Datum` to `(title, value)` pairs. Times are written as ISO 8601 in the
//! zone of the data, so nothing in the text depends on the machine.

use crate::data::{Cell, Datum, time::Zone};
use crate::labels::Labels;
use crate::scale::format::{iso, plain};
use crate::spec::{Aggregate, Channel, ChannelDef, FieldType, Spec};

const ORDER: [Channel; 4] = [Channel::X, Channel::Y, Channel::Theta, Channel::Color];

/// The header of the column a channel makes.
pub fn heading(def: &ChannelDef, l: &Labels) -> String {
    let f = def.field.clone().unwrap_or_default();
    let t = match def.aggregate {
        None => return f,
        Some(Aggregate::Count) => return l.count_title.clone(),
        Some(Aggregate::Sum) => &l.sum_title,
        Some(Aggregate::Mean) => &l.mean_title,
        Some(Aggregate::Min) => &l.min_title,
        Some(Aggregate::Max) => &l.max_title,
    };
    Labels::fill(t, &[("field", &f)])
}

/// The value of one cell as text.
pub fn value(cell: &Cell, def: &ChannelDef, zone: Zone, l: &Labels) -> String {
    match cell {
        Cell::Num(n) if def.ftype == FieldType::Temporal => iso(*n, zone),
        Cell::Num(n) => plain(*n, l),
        Cell::Text(t) => t.clone(),
        Cell::Null if def.ftype.is_discrete() => l.null_category.clone(),
        _ => String::new(),
    }
}

/// `(heading, value)` for each channel the spec uses.
pub fn pairs(spec: &Spec, d: &Datum, zones: &[Zone; 4], l: &Labels) -> Vec<(String, String)> {
    ORDER
        .iter()
        .filter_map(|&c| {
            let def = spec.encoding.get(c)?;
            Some((
                heading(def, l),
                value(&d.cells[c.index()], def, zones[c.index()], l),
            ))
        })
        .collect()
}

/// The pairs as lines `heading: value`.
pub fn text(spec: &Spec, d: &Datum, zones: &[Zone; 4], l: &Labels) -> String {
    pairs(spec, d, zones, l)
        .into_iter()
        .map(|(h, v)| format!("{h}: {v}"))
        .collect::<Vec<_>>()
        .join("\n")
}
