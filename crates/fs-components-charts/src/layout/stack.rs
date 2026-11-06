//! Stacking: where each value starts and ends when values of one group pile up.
//!
//! Pattern: Pure function over `(group, rank, value)`. Positive values pile up from zero and
//! negative values pile down from zero, separately; with `Normalize` Vega-Lite uses the absolute
//! values (checked against 6.5.0), so every pile is 0..1. The order of a pile is by the natural
//! rank of the series, not by `sort`: ascending from zero for a horizontal bar and a pie,
//! descending (the first series ends on top) for a vertical bar and an area.

use crate::spec::StackMode;

/// One value to stack: its group (a category or an x position), its series rank, and the value.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entry {
    /// Which pile.
    pub group: usize,
    /// Natural rank of the series (`Cats::rank`).
    pub rank: usize,
    /// The value.
    pub value: f64,
}

impl Entry {
    /// An entry.
    pub fn new(group: usize, rank: usize, value: f64) -> Self {
        Self { group, rank, value }
    }
}

/// `[low, high]` for each entry, in the order given. `Off` gives `[0, value]` for all.
/// `from_zero` puts the lowest rank at the zero end.
pub fn stack(entries: &[Entry], mode: StackMode, from_zero: bool) -> Vec<[f64; 2]> {
    if mode == StackMode::Off {
        return entries
            .iter()
            .map(|e| [0.0_f64.min(e.value), 0.0_f64.max(e.value)])
            .collect();
    }
    let norm = mode == StackMode::Normalize;
    let mut order: Vec<usize> = (0..entries.len()).collect();
    order.sort_by_key(|&i| {
        let e = &entries[i];
        let rank = if from_zero {
            e.rank
        } else {
            usize::MAX - e.rank
        };
        (e.group, rank, i)
    });
    let mut out = vec![[0.0, 0.0]; entries.len()];
    // totals per group, for normalising; scaled by the largest value so they cannot overflow
    let mut scale = 1.0_f64;
    let mut totals: Vec<(usize, f64)> = Vec::new();
    if norm {
        scale = entries
            .iter()
            .map(|e| e.value.abs())
            .fold(0.0, f64::max)
            .max(f64::MIN_POSITIVE);
        for &i in &order {
            let e = &entries[i];
            if totals.last().map(|t| t.0) != Some(e.group) {
                totals.push((e.group, 0.0));
            }
            if let Some(t) = totals.last_mut() {
                t.1 += e.value.abs() / scale;
            }
        }
    }
    let (mut pos, mut neg, mut current) = (0.0_f64, 0.0_f64, None);
    let mut pile = 0;
    for &i in &order {
        let e = &entries[i];
        if current != Some(e.group) {
            current = Some(e.group);
            (pos, neg) = (0.0, 0.0);
            pile = totals.iter().position(|t| t.0 == e.group).unwrap_or(0);
        }
        if norm {
            let total = totals.get(pile).map_or(0.0, |t| t.1);
            let part = if total > 0.0 {
                e.value.abs() / scale / total
            } else {
                0.0
            };
            out[i] = [pos, pos + part];
            pos += part;
            continue;
        }
        let (low, high) = if e.value >= 0.0 {
            (pos, pos + e.value)
        } else {
            (neg + e.value, neg)
        };
        if e.value >= 0.0 {
            pos = high;
        } else {
            neg = low;
        }
        out[i] = [low.max(-f64::MAX), high.min(f64::MAX)];
    }
    out
}
