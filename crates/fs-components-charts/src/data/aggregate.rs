//! Grouping rows and reducing a group to one number.
//!
//! Pattern: Strategy by aggregate — one small function per operation; the grouping key is every
//! channel that is *not* aggregated, as in Vega-Lite.

use std::collections::HashMap;
use std::fmt::Write;

use super::cell::{Cell, Invalid};
use super::prepare::Datum;
use crate::spec::{Aggregate, ChannelDef, Encoding};

const CHANNELS: usize = 4;

/// Group `rows` by the plain channels and reduce the aggregated ones; order of first appearance.
pub(super) fn group(enc: &Encoding, rows: Vec<Datum>) -> Vec<Datum> {
    let defs = defs(enc);
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut groups: Vec<Vec<Datum>> = Vec::new();
    for row in rows {
        let mut key = String::new();
        for (cell, def) in row.cells.iter().zip(&defs) {
            if def.is_some_and(|d| d.aggregate.is_none()) {
                let _ = write!(key, "{cell:?}\u{1}");
            }
        }
        let slot = *index.entry(key).or_insert_with(|| {
            groups.push(Vec::new());
            groups.len() - 1
        });
        groups[slot].push(row);
    }
    groups
        .into_iter()
        .map(|g| {
            let mut out = g[0].clone();
            for (c, def) in defs.iter().enumerate() {
                if let Some(a) = def.and_then(|d| d.aggregate) {
                    out.cells[c] = reduce(a, g.iter().map(|d| &d.cells[c]), g.len());
                }
            }
            out
        })
        .collect()
}

fn defs(enc: &Encoding) -> [Option<&ChannelDef>; CHANNELS] {
    [
        enc.x.as_ref(),
        enc.y.as_ref(),
        enc.color.as_ref(),
        enc.theta.as_ref(),
    ]
}

/// One group's value. `count` counts rows; the others use the valid numbers only.
fn reduce<'a>(a: Aggregate, cells: impl Iterator<Item = &'a Cell>, n: usize) -> Cell {
    #[allow(clippy::cast_precision_loss)]
    if a == Aggregate::Count {
        return Cell::Num(n as f64);
    }
    let v: Vec<f64> = cells.filter_map(Cell::num).collect();
    let out = match a {
        Aggregate::Sum if !v.is_empty() => Some(v.iter().sum::<f64>() + 0.0),
        #[allow(clippy::cast_precision_loss)]
        Aggregate::Mean if !v.is_empty() => Some(mean(&v)),
        Aggregate::Min => v.iter().copied().reduce(f64::min),
        Aggregate::Max => v.iter().copied().reduce(f64::max),
        _ => None,
    };
    match out {
        Some(x) if x.is_finite() => Cell::Num(x),
        Some(_) => Cell::Invalid(Invalid::Overflow),
        None => Cell::Null,
    }
}

/// Mean without overflowing on huge values: each term is divided first.
#[allow(clippy::cast_precision_loss)]
fn mean(v: &[f64]) -> f64 {
    let n = v.len() as f64;
    v.iter().map(|x| x / n).sum()
}
