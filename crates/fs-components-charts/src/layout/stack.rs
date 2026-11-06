//! Stacking: where each value starts and ends when values of one group pile up.
//!
//! Pattern: Pure function over `(group, series, value)`. As in Vega-Lite, series 0 ends up on top
//! (the stack is built from the highest series number), positive values pile up from zero and
//! negative values pile down from zero, separately.

use crate::spec::StackMode;

/// One value to stack: its group (a category or an x position), its series, and the value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entry {
    /// Which pile.
    pub group: usize,
    /// Which series (colour).
    pub series: usize,
    /// The value.
    pub value: f64,
}

/// `[low, high]` for each entry, in the order given. `Off` gives `[0, value]` for all.
pub fn stack(entries: &[Entry], mode: StackMode) -> Vec<[f64; 2]> {
    if mode == StackMode::Off {
        return entries
            .iter()
            .map(|e| [0.0_f64.min(e.value), 0.0_f64.max(e.value)])
            .collect();
    }
    let mut order: Vec<usize> = (0..entries.len()).collect();
    order.sort_by(|&a, &b| {
        (entries[a].group, std::cmp::Reverse(entries[a].series), a).cmp(&(
            entries[b].group,
            std::cmp::Reverse(entries[b].series),
            b,
        ))
    });
    let mut out = vec![[0.0, 0.0]; entries.len()];
    let (mut pos, mut neg, mut current) = (0.0_f64, 0.0_f64, None);
    let mut piles: Vec<(usize, f64)> = Vec::new(); // (group, total height) for normalising
    for &i in &order {
        let e = &entries[i];
        if current != Some(e.group) {
            current = Some(e.group);
            (pos, neg) = (0.0, 0.0);
            piles.push((e.group, 0.0));
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
        if let Some(last) = piles.last_mut() {
            last.1 += e.value.abs();
        }
    }
    if mode == StackMode::Normalize {
        let total = |g: usize| piles.iter().find(|p| p.0 == g).map_or(1.0, |p| p.1);
        for (e, o) in entries.iter().zip(out.iter_mut()) {
            let t = total(e.group);
            *o = if t > 0.0 {
                [o[0] / t, o[1] / t]
            } else {
                [0.0, 0.0]
            };
        }
    }
    out
}
