//! Tests of the pure model: scale, rectangles, ticks, routes, zoom, keys. No window.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;

use fs_components_timeline::layout::{
    Rect, Rows, Scale, TickLabel, item_rect, month_marks, progress_rect, range, ticks,
};
use fs_components_timeline::route::{GAP, route};
use fs_components_timeline::*;
use fs_components_widgets::Key;
use host::{d, items};

const G: Rows = Rows {
    row: 36.0,
    bar: 24.0,
};

#[test]
fn days_map_to_pixels_and_back() {
    let s = Scale {
        origin: d(2026, 11, 1),
        px: 12.0,
    };
    assert!((s.x_of(d(2026, 11, 3)) - 24.0).abs() < 0.01);
    assert_eq!(s.day_at(25.0), d(2026, 11, 3));
    assert_eq!(s.day_at(23.9), d(2026, 11, 2));
}

#[test]
fn the_range_covers_all_items_with_padding() {
    let (a, b) = range(&items(), 7, d(2026, 1, 1));
    assert_eq!((a, b), (d(2026, 10, 26), d(2026, 12, 21)));
    assert_eq!(
        range(&[], 3, d(2026, 1, 10)),
        (d(2026, 1, 7), d(2026, 1, 13))
    );
}

#[test]
fn a_bar_covers_its_last_day_a_milestone_is_a_square_on_its_day_and_progress_is_a_share() {
    let s = Scale {
        origin: d(2026, 11, 1),
        px: 10.0,
    };
    let it = items();
    let bar = item_rect(&it[0], 0, &s, G);
    assert_eq!(
        (bar.x, bar.w, bar.h),
        (10.0, 120.0, 24.0),
        "2 to 13 November, both days inclusive"
    );
    assert_eq!(bar.y, 6.0, "centred in its row");
    let low = item_rect(&it[1], 1, &s, G);
    assert_eq!(low.y, 36.0 + 6.0, "second row");
    let m = item_rect(&it[3], 3, &s, G);
    assert_eq!(m.w, m.h);
    assert!(
        (m.x + m.w / 2.0 - (s.x_of(d(2026, 12, 14)) + 5.0)).abs() < 0.01,
        "centred on the day"
    );
    assert_eq!(progress_rect(bar, 25).w, 30.0);
    assert_eq!(progress_rect(bar, 200).w, 120.0, "never more than the bar");
}

#[test]
fn ticks_follow_the_zoom() {
    let (a, b) = (d(2026, 11, 25), d(2026, 12, 8));
    let days = ticks(Zoom::Day, a, b);
    assert_eq!(days.len(), 14);
    assert_eq!(days[0].label, TickLabel::Day(25));
    let weeks = ticks(Zoom::Week, a, b);
    assert_eq!(
        weeks.iter().map(|t| t.label).collect::<Vec<_>>(),
        [TickLabel::Day(30), TickLabel::Day(7)],
        "Mondays"
    );
    let months = ticks(Zoom::Month, a, b);
    assert_eq!(months.len(), 2);
    assert_eq!(
        (months[0].label, months[0].days),
        (TickLabel::Month(11, 2026), 6),
        "the rest of November"
    );
    assert_eq!(months[1].label, TickLabel::Month(12, 2026));
    assert_eq!(month_marks(a, b), months);
}

#[test]
fn a_dependency_runs_right_down_and_into_the_next_item_with_a_head() {
    let a = Rect {
        x: 0.0,
        y: 6.0,
        w: 100.0,
        h: 24.0,
    };
    let b = Rect {
        x: 140.0,
        y: 42.0,
        w: 80.0,
        h: 24.0,
    };
    let r = route(a, b, 36.0);
    assert_eq!(r.line.first(), Some(&(100.0, 18.0)));
    assert_eq!(r.line.last(), Some(&(140.0, 54.0)));
    assert_eq!(r.line[1], (100.0 + GAP, 18.0));
    assert_eq!(r.head[0], (140.0, 54.0), "the tip is at the next item");
    let same = route(
        a,
        Rect {
            x: 140.0,
            y: 6.0,
            ..b
        },
        36.0,
    );
    assert_eq!(same.line.len(), 2, "straight on one row");
}

#[test]
fn a_dependency_to_an_item_that_starts_earlier_goes_round_between_the_rows() {
    let a = Rect {
        x: 100.0,
        y: 6.0,
        w: 100.0,
        h: 24.0,
    };
    let b = Rect {
        x: 120.0,
        y: 42.0,
        w: 80.0,
        h: 24.0,
    };
    let r = route(a, b, 36.0);
    assert_eq!(r.line.len(), 6);
    let mid = r.line[2].1;
    assert!(
        mid > 18.0 && mid < 54.0,
        "between the two rows' centres: {mid}"
    );
    assert!(r.line[3].0 < b.x, "comes in from the left of the item");
}

#[test]
fn zoom_steps_stop_at_the_ends_and_widths_differ() {
    assert_eq!(Zoom::Day.finer(), Zoom::Day);
    assert_eq!(Zoom::Month.coarser(), Zoom::Month);
    assert_eq!(Zoom::Week.coarser(), Zoom::Month);
    assert!(
        Zoom::Day.px_per_day() > Zoom::Week.px_per_day()
            && Zoom::Week.px_per_day() > Zoom::Month.px_per_day()
    );
}

#[test]
fn keys_walk_the_rows_and_zoom() {
    let it = items();
    let mut s = TimelineState::new(None);
    assert_eq!(s.key(&it, Key::Down), Step::Select("design".into()));
    assert_eq!(s.key(&it, Key::Down), Step::Select("build".into()));
    assert_eq!(s.key(&it, Key::Up), Step::Select("design".into()));
    assert_eq!(s.key(&it, Key::Up), Step::Ignored, "stays at the top");
    assert_eq!(s.key(&it, Key::End), Step::Select("ship".into()));
    assert_eq!(s.key(&it, Key::Other(187)), Step::Zoomed(Zoom::Day));
    assert_eq!(s.key(&it, Key::Other(187)), Step::Ignored);
    assert_eq!(s.key(&it, Key::Other(109)), Step::Zoomed(Zoom::Week));
    assert_eq!(s.key(&it, Key::Other(65)), Step::Ignored);
    assert_eq!(s.key(&[], Key::Down), Step::Ignored, "no rows");
}
