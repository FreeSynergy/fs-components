//! The time axis: the part of an item on one day, side-by-side lanes, minutes to pixels.
//!
//! Pattern: none — pure functions; testable without a window.

use crate::day::{Day, MINUTES_PER_DAY};
use crate::model::GridItem;

/// The part of an item on one day, in minutes of that day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segment {
    /// Index into the item list.
    pub index: usize,
    /// Begin.
    pub start: u16,
    /// End, not before `start`.
    pub end: u16,
}

/// Timed items shorter than a day that are on `day`.
pub fn day_segments(items: &[GridItem], day: Day) -> Vec<Segment> {
    items
        .iter()
        .enumerate()
        .filter(|(_, i)| !i.is_span() && i.start.day == day)
        .map(|(index, i)| Segment {
            index,
            start: i.start.minute,
            end: i.end.minute.max(i.start.minute).min(MINUTES_PER_DAY),
        })
        .collect()
}

/// A segment with its lane among those it overlaps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    /// The segment.
    pub segment: Segment,
    /// Lane from 0.
    pub lane: usize,
    /// Lanes in the overlapping group.
    pub lanes: usize,
}

/// Side-by-side lanes. A segment counts as at least `shortest` minutes, so a short item next to
/// another does not hide under it.
pub fn lanes(segments: &[Segment], shortest: u16) -> Vec<Placed> {
    let reach = |s: &Segment| s.end.max(s.start.saturating_add(shortest));
    let mut order: Vec<Segment> = segments.to_vec();
    order.sort_by_key(|s| (s.start, std::cmp::Reverse(reach(s)), s.index));
    let mut placed: Vec<Placed> = Vec::new();
    let (mut group_start, mut group_end) = (0, 0);
    let mut ends: Vec<u16> = Vec::new();
    for s in order {
        if !ends.is_empty() && s.start >= group_end {
            for p in &mut placed[group_start..] {
                p.lanes = ends.len();
            }
            group_start = placed.len();
            ends.clear();
            group_end = 0;
        }
        let lane = ends
            .iter()
            .position(|&e| e <= s.start)
            .unwrap_or(ends.len());
        if lane == ends.len() {
            ends.push(0);
        }
        ends[lane] = reach(&s);
        group_end = group_end.max(reach(&s));
        placed.push(Placed {
            segment: s,
            lane,
            lanes: 0,
        });
    }
    for p in &mut placed[group_start..] {
        p.lanes = ends.len().max(1);
    }
    placed
}

/// Pixel offset of `minute` on an axis with `hour_px` pixels per hour.
pub fn y_of(minute: u16, hour_px: f32) -> f32 {
    f32::from(minute) / 60.0 * hour_px
}

/// The minute at pixel `y`, snapped down to a multiple of `snap` minutes and kept inside the day.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped to the day first
pub fn minute_at(y: f32, hour_px: f32, snap: u16) -> u16 {
    let raw = (y / hour_px * 60.0).clamp(0.0, f32::from(MINUTES_PER_DAY - 1)) as u16;
    let snap = snap.max(1);
    raw / snap * snap
}
