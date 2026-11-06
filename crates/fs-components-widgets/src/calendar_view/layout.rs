//! Where things go: pure functions, testable without a window.
//!
//! Pattern: Strategy-free pure layout — the spans of the views, an item cut into the part on one day,
//! and overlapping parts split sideways into lanes (the docs: split sideways, never stack).

use super::day::{Day, MINUTES_PER_DAY};
use super::model::{CalendarItem, ViewRange};

/// The first day of the week holding `day`; `week_start` is 0 for Monday to 6 for Sunday.
pub fn week_of(day: Day, week_start: u8) -> Day {
    let back = (i64::from(day.weekday()) - i64::from(week_start)).rem_euclid(7);
    day.add_days(-back)
}

/// The six weeks shown for the month of `anchor`.
pub fn month_days(anchor: Day, week_start: u8) -> Vec<Day> {
    let first = week_of(anchor.first_of_month(), week_start);
    (0..42).map(|i| first.add_days(i)).collect()
}

/// First and last day a view of `range` around `anchor` shows. An agenda shows `agenda_days` from
/// `anchor`.
pub fn span_of(range: ViewRange, anchor: Day, week_start: u8, agenda_days: u16) -> (Day, Day) {
    match range {
        ViewRange::Month => {
            let first = week_of(anchor.first_of_month(), week_start);
            (first, first.add_days(41))
        }
        ViewRange::Week => {
            let first = week_of(anchor, week_start);
            (first, first.add_days(6))
        }
        ViewRange::Agenda => (anchor, anchor.add_days(i64::from(agenda_days.max(1)) - 1)),
        _ => (anchor, anchor),
    }
}

/// The anchor one step on (back, if `forward` is false).
pub fn step_from(range: ViewRange, anchor: Day, forward: bool, agenda_days: u16) -> Day {
    let sign = if forward { 1 } else { -1 };
    match range {
        ViewRange::Month => anchor.add_months(sign),
        ViewRange::Week => anchor.add_days(i64::from(sign) * 7),
        ViewRange::Agenda => anchor.add_days(i64::from(sign) * i64::from(agenda_days.max(1))),
        _ => anchor.add_days(i64::from(sign)),
    }
}

/// The part of an item on one day, in minutes of that day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segment {
    /// Begins here.
    pub start: u16,
    /// Ends here, not before `start`; up to [`MINUTES_PER_DAY`].
    pub end: u16,
}

/// The part of `item` on `day`, if it has one. An item ending at midnight is not on the next day; one
/// of no length is on the day it is at.
pub fn segment_on(item: &CalendarItem, day: Day) -> Option<Segment> {
    let (start, end) = (item.start, item.end);
    let empty = start == end;
    let on = start.day <= day && (day < end.day || (day == end.day && (end.minute > 0 || empty)));
    if !on {
        return None;
    }
    let from = if start.day == day { start.minute } else { 0 };
    let to = if end.day > day {
        MINUTES_PER_DAY
    } else {
        end.minute
    };
    Some(Segment {
        start: from,
        end: to.max(from),
    })
}

/// A segment with its place among those it overlaps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    /// Which input it is.
    pub index: usize,
    /// Its lane, from 0.
    pub lane: usize,
    /// How many lanes the overlapping group has.
    pub lanes: usize,
}

/// Lanes for `segments`. A segment takes at least `shortest` minutes, what it needs to be read, so a
/// short item next to another does not end up under it. Those overlapping are side by side; a lane
/// is used again as soon as it is free, and a group ends when nothing reaches into the next segment.
pub fn lanes(segments: &[Segment], shortest: u16) -> Vec<Placed> {
    let reach = |s: &Segment| s.end.max(s.start.saturating_add(shortest));
    let mut order: Vec<usize> = (0..segments.len()).collect();
    order.sort_by_key(|&i| (segments[i].start, std::cmp::Reverse(reach(&segments[i])), i));
    let mut placed: Vec<Placed> = Vec::with_capacity(segments.len());
    let mut group_start = 0;
    let mut group_end = 0;
    let mut lane_ends: Vec<u16> = Vec::new();
    for index in order {
        let segment = &segments[index];
        if !lane_ends.is_empty() && segment.start >= group_end {
            close_group(&mut placed[group_start..], lane_ends.len());
            group_start = placed.len();
            lane_ends.clear();
            group_end = 0;
        }
        let free = lane_ends.iter().position(|end| *end <= segment.start);
        let lane = free.unwrap_or(lane_ends.len());
        if lane == lane_ends.len() {
            lane_ends.push(0);
        }
        lane_ends[lane] = reach(segment);
        group_end = group_end.max(reach(segment));
        placed.push(Placed {
            index,
            lane,
            lanes: 0,
        });
    }
    close_group(&mut placed[group_start..], lane_ends.len());
    placed
}

fn close_group(group: &mut [Placed], lanes: usize) {
    for placed in group {
        placed.lanes = lanes;
    }
}
