//! Grouping on the timeline.
//!
//! Pattern: Strategy — [`Grouping`] picks how much of the date names the group; the groups are a pure function of the
//! pictures. The label is the leading part of the RFC 3339 text (`2026-11-03`, `2026-11`, `2026`), which sorts
//! and reads the same in every language; a frontend that wants a month name maps the label itself.

use crate::Picture;

/// How the timeline is cut.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grouping {
    /// One group per day.
    Day,
    /// One group per month.
    Month,
    /// One group per year.
    Year,
}

/// The group a time belongs to; empty when the time is not known or not a date.
pub fn group_label(taken: &str, by: Grouping) -> String {
    let len = match by {
        Grouping::Day => 10,
        Grouping::Month => 7,
        Grouping::Year => 4,
    };
    let head: String = taken.chars().take(len).collect();
    let digits_ok = head.len() == len
        && head.char_indices().all(|(i, c)| {
            if matches!(i, 4 | 7) {
                c == '-'
            } else {
                c.is_ascii_digit()
            }
        });
    if digits_ok { head } else { String::new() }
}

/// The pictures of one group, newest first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    /// The group label; empty for the pictures without a date, which come last.
    pub label: String,
    /// The pictures, newest first, ties by name then key.
    pub pictures: Vec<Picture>,
}

/// The groups, newest first; pictures without a date form the last group.
pub fn sections(pictures: Vec<Picture>, by: Grouping) -> Vec<Section> {
    let mut all = pictures;
    all.sort_by(|a, b| {
        b.taken
            .cmp(&a.taken)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.key.cmp(&b.key))
    });
    let mut out: Vec<Section> = Vec::new();
    for p in all {
        let label = group_label(&p.taken, by);
        match out.iter_mut().find(|s| s.label == label) {
            Some(s) => s.pictures.push(p),
            None => out.push(Section {
                label,
                pictures: vec![p],
            }),
        }
    }
    // pictures without a date are sorted last by their empty text; make that explicit
    out.sort_by_key(|s| s.label.is_empty());
    out
}
