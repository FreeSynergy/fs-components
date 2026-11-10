//! Bars over several columns: all-day items and items lasting more than one day, in lanes.
//!
//! Pattern: none — a pure function; a greedy lane packing per row of days, longer bars first so
//! they stay on top, as every calendar does it.

use crate::day::Day;
use crate::model::GridItem;

/// One bar in a row of days.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpanBar {
    /// Index into the item list.
    pub index: usize,
    /// First column (0-based) in this row.
    pub first: usize,
    /// Last column, inclusive.
    pub last: usize,
    /// Lane (line from the top), 0-based.
    pub lane: usize,
    /// The item began before this row.
    pub cut_left: bool,
    /// The item goes on after this row.
    pub cut_right: bool,
}

/// Bars for each row of `cols` days taken from `days` in order. `only_spans` leaves out items that
/// are shorter than a day and timed (they belong to a time column).
pub fn span_rows(
    items: &[GridItem],
    days: &[Day],
    cols: usize,
    only_spans: bool,
) -> Vec<Vec<SpanBar>> {
    days.chunks(cols)
        .map(|row| one_row(items, row, only_spans))
        .collect()
}

fn one_row(items: &[GridItem], row: &[Day], only_spans: bool) -> Vec<SpanBar> {
    let (Some(&from), Some(&to)) = (row.first(), row.last()) else {
        return Vec::new();
    };
    let mut bars: Vec<SpanBar> = Vec::new();
    for (index, item) in items.iter().enumerate() {
        if only_spans && !item.is_span() {
            continue;
        }
        let (s, e) = (item.start.day, item.last_day());
        if s > to || e < from {
            continue;
        }
        let col = |d: Day| usize::try_from(d.0 - from.0).unwrap_or(0);
        bars.push(SpanBar {
            index,
            first: if s < from { 0 } else { col(s) },
            last: if e > to { row.len() - 1 } else { col(e) },
            lane: 0,
            cut_left: s < from,
            cut_right: e > to,
        });
    }
    bars.sort_by_key(|b| (b.first, std::cmp::Reverse(b.last - b.first), b.index));
    let mut lane_ends: Vec<usize> = Vec::new(); // last column used, per lane
    for bar in &mut bars {
        let free = lane_ends.iter().position(|&end| end < bar.first);
        bar.lane = free.unwrap_or(lane_ends.len());
        if bar.lane == lane_ends.len() {
            lane_ends.push(0);
        }
        lane_ends[bar.lane] = bar.last;
    }
    bars
}

/// Lines the bars of a row need.
pub fn lanes_used(bars: &[SpanBar]) -> usize {
    bars.iter().map(|b| b.lane + 1).max().unwrap_or(0)
}
