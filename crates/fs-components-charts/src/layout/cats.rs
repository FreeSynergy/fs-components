//! The categories of a discrete channel: which values, in which order.
//!
//! Pattern: Value object built by a Strategy per sort mode. The order is Vega-Lite's: ascending
//! by default (numbers before text, `null` last), or as the data came, or descending, or an
//! explicit list, or by the total of the other axis.

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::data::{Cell, Datum};
use crate::labels::Labels;
use crate::scale::format::plain;
use crate::spec::{Channel, ChannelDef, Sort};

/// Ordered category names and where each one is.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cats {
    /// Names in domain order.
    pub names: Vec<String>,
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

    fn from(names: Vec<String>) -> Self {
        let index = names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), i))
            .collect();
        Self { names, index }
    }
}

/// The name of a category value.
pub fn key(cell: &Cell, labels: &Labels) -> String {
    match cell {
        Cell::Text(t) => t.clone(),
        Cell::Num(n) => plain(*n, labels),
        _ => labels.null_category.clone(),
    }
}

fn rank(c: &Cell) -> u8 {
    match c {
        Cell::Num(_) => 0,
        Cell::Text(_) => 1,
        _ => 2,
    }
}

fn natural(a: &Cell, b: &Cell) -> Ordering {
    match (a, b) {
        (Cell::Num(x), Cell::Num(y)) => x.total_cmp(y),
        (Cell::Text(x), Cell::Text(y)) => x.cmp(y),
        _ => rank(a).cmp(&rank(b)),
    }
}

/// The categories of `channel` in the data, ordered by the channel's `sort`.
pub fn categories(def: &ChannelDef, channel: Channel, data: &[Datum], labels: &Labels) -> Cats {
    let i = channel.index();
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut firsts: Vec<(&Cell, f64)> = Vec::new();
    for d in data {
        let cell = &d.cells[i];
        let k = key(cell, labels);
        let slot = *seen.entry(k).or_insert_with(|| {
            firsts.push((cell, 0.0));
            firsts.len() - 1
        });
        if let Sort::ByChannel { channel: other, .. } = def.sort {
            firsts[slot].1 += d.cells[other.index()].num().unwrap_or(0.0);
        }
    }
    match &def.sort {
        Sort::DataOrder => {}
        Sort::Ascending => firsts.sort_by(|a, b| natural(a.0, b.0)),
        Sort::Descending => firsts.sort_by(|a, b| natural(b.0, a.0)),
        Sort::ByChannel { descending, .. } => firsts.sort_by(|a, b| {
            let o = a.1.total_cmp(&b.1).then_with(|| natural(a.0, b.0));
            if *descending { o.reverse() } else { o }
        }),
        Sort::Values(list) => {
            let pos = |c: &Cell| {
                let k = key(c, labels);
                list.iter()
                    .position(|v| listed(v, labels) == k)
                    .unwrap_or(usize::MAX)
            };
            firsts.sort_by(|a, b| pos(a.0).cmp(&pos(b.0)).then_with(|| natural(a.0, b.0)));
        }
    }
    Cats::from(firsts.iter().map(|(c, _)| key(c, labels)).collect())
}

fn listed(v: &serde_json::Value, labels: &Labels) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.as_f64().map_or_else(String::new, |x| plain(x, labels)),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => labels.null_category.clone(),
    }
}
