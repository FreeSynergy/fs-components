//! The categories of a discrete channel: which values, in which order.
//!
//! Pattern: Strategy. The order is Vega-Lite's: ascending
//! by default (numbers before text, `null` last), or as the data came, or descending, or an
//! explicit list, or by the total of the other axis.

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::data::{Cell, Datum};
use crate::labels::Labels;
use crate::scale::format::plain;
use crate::spec::{Aggregate, Channel, ChannelDef, Sort};

/// Ordered category names and where each one is.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cats {
    /// Names in domain order.
    pub names: Vec<String>,
    /// For each name: its place in the *natural* order of the values (ascending), whatever the
    /// `sort` says. Vega-Lite stacks and cuts a pie in this order; `sort` only moves the axis,
    /// the colours and the legend.
    pub rank: Vec<usize>,
    index: HashMap<String, usize>,
}

impl Cats {
    /// Position of the category of `cell`.
    pub fn index_of(&self, cell: &Cell, labels: &Labels) -> Option<usize> {
        self.index.get(&key(cell, labels)).copied()
    }

    /// Number of categories.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// No category.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    fn from(names: Vec<String>, rank: Vec<usize>) -> Self {
        let index = names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), i))
            .collect();
        Self { names, rank, index }
    }
}

/// The name of a category value.
pub fn key(cell: &Cell, labels: &Labels) -> String {
    match cell {
        Cell::Text(t) => t.clone(),
        Cell::Num(n) => plain(*n, labels),
        Cell::Absent => labels.absent_category.clone(),
        _ => labels.null_category.clone(),
    }
}

/// `null` first, then a missing field, then numbers, then text (as Vega-Lite orders them).
fn rank(c: &Cell) -> u8 {
    match c {
        Cell::Absent => 1,
        Cell::Num(_) => 2,
        Cell::Text(_) => 3,
        _ => 0,
    }
}

fn natural(a: &Cell, b: &Cell) -> Ordering {
    match (a, b) {
        (Cell::Num(x), Cell::Num(y)) => x.total_cmp(y),
        (Cell::Text(x), Cell::Text(y)) => x.cmp(y),
        _ => rank(a).cmp(&rank(b)),
    }
}

/// What a sort by the other axis needs: that axis' definition and whether it is stacked (a bar
/// or area value axis).
#[derive(Clone, Copy)]
pub struct Other<'a> {
    /// The other positional channel.
    pub def: &'a ChannelDef,
    /// Its values pile up.
    pub stacked: bool,
}

/// The one number a category is sorted by: the channel's own aggregate if it has one, else the
/// sum for a stacked axis and the smallest value otherwise (what Vega-Lite 6.5.0 does).
fn reduce(values: &[f64], other: Other<'_>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let sum: f64 = values.iter().sum();
    Some(match other.def.aggregate {
        Some(Aggregate::Mean) => sum / values.len() as f64,
        Some(Aggregate::Min) => values.iter().copied().fold(f64::INFINITY, f64::min),
        Some(Aggregate::Max) => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        Some(_) => sum,
        None if other.stacked => sum,
        None => values.iter().copied().fold(f64::INFINITY, f64::min),
    })
}

/// The categories of `channel` in the data, ordered by the channel's `sort`.
pub fn categories(
    def: &ChannelDef,
    channel: Channel,
    data: &[Datum],
    labels: &Labels,
    other: Option<Other<'_>>,
) -> Cats {
    let i = channel.index();
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut firsts: Vec<(&Cell, Vec<f64>)> = Vec::new();
    for d in data {
        let cell = &d.cells[i];
        let slot = *seen.entry(key(cell, labels)).or_insert_with(|| {
            firsts.push((cell, Vec::new()));
            firsts.len() - 1
        });
        if let (Sort::ByChannel { channel: o, .. }, Some(_)) = (&def.sort, other)
            && let Some(v) = d.cells[o.index()].num()
        {
            firsts[slot].1.push(v);
        }
    }
    // natural rank of each value, by first appearance
    let mut by_nature: Vec<usize> = (0..firsts.len()).collect();
    by_nature.sort_by(|&a, &b| natural(firsts[a].0, firsts[b].0));
    let mut nat = vec![0; firsts.len()];
    for (r, &idx) in by_nature.iter().enumerate() {
        nat[idx] = r;
    }
    let mut order: Vec<usize> = (0..firsts.len()).collect();
    match (&def.sort, other) {
        (Sort::DataOrder, _) => {}
        (Sort::Descending, _) => order.sort_by(|&a, &b| natural(firsts[b].0, firsts[a].0)),
        (Sort::ByChannel { descending, .. }, Some(o)) => {
            let sums: Vec<Option<f64>> = firsts.iter().map(|f| reduce(&f.1, o)).collect();
            order.sort_by(|&a, &b| {
                let by_value = match (sums[a], sums[b]) {
                    (Some(x), Some(y)) if *descending => y.total_cmp(&x),
                    (Some(x), Some(y)) => x.total_cmp(&y),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => Ordering::Equal,
                };
                by_value.then_with(|| nat[a].cmp(&nat[b]))
            });
        }
        (Sort::Values(list), _) => {
            // listed values in the list's order; the rest after them, in the order of the data
            let pos = |c: &Cell| {
                let k = key(c, labels);
                list.iter()
                    .position(|v| listed(v, labels) == k)
                    .unwrap_or(usize::MAX)
            };
            order.sort_by_key(|&a| pos(firsts[a].0));
        }
        _ => order.sort_by(|&a, &b| nat[a].cmp(&nat[b])),
    }
    let names = order.iter().map(|&k| key(firsts[k].0, labels)).collect();
    Cats::from(names, order.iter().map(|&k| nat[k]).collect())
}

fn listed(v: &serde_json::Value, labels: &Labels) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.as_f64().map_or_else(String::new, |x| plain(x, labels)),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => labels.null_category.clone(),
    }
}
