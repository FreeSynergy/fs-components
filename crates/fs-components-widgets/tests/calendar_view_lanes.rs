//! `calendar-view`: overlapping items split sideways into lanes. Pure, no window.
//!
//! Pattern: Table tests over the layout function.

use fs_components_widgets::calendar_view::{Segment, lanes};

fn seg(start: u16, end: u16) -> Segment {
    Segment { start, end }
}

fn placed(segments: &[Segment], shortest: u16) -> Vec<(usize, usize)> {
    let mut places = lanes(segments, shortest);
    places.sort_by_key(|p| p.index);
    places.iter().map(|p| (p.lane, p.lanes)).collect()
}

#[test]
fn what_overlaps_goes_side_by_side_and_the_rest_keeps_the_whole_width() {
    assert_eq!(placed(&[seg(60, 120), seg(180, 240)], 30), [(0, 1), (0, 1)]);
    assert_eq!(placed(&[seg(60, 180), seg(120, 240)], 30), [(0, 2), (1, 2)]);
    assert_eq!(
        placed(&[seg(60, 300), seg(60, 120), seg(120, 180)], 30),
        [(0, 2), (1, 2), (1, 2)],
        "a lane that is free is used again"
    );
    assert_eq!(
        placed(&[seg(60, 120), seg(120, 180)], 30),
        [(0, 1), (0, 1)],
        "one ending when the other begins does not overlap"
    );
}

#[test]
fn a_short_item_keeps_the_room_it_needs_to_be_read() {
    assert_eq!(
        placed(&[seg(60, 65), seg(70, 90)], 30),
        [(0, 2), (1, 2)],
        "five minutes are drawn as thirty"
    );
    assert_eq!(placed(&[seg(60, 60), seg(60, 60)], 30), [(0, 2), (1, 2)]);
}

#[test]
fn nothing_is_lost_in_a_crowd() {
    let many: Vec<Segment> = (0..50).map(|i| seg(60 + i, 200)).collect();
    let places = lanes(&many, 30);
    assert_eq!(places.len(), 50);
    let mut seen: Vec<usize> = places.iter().map(|p| p.lane).collect();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), 50, "each in a lane of its own");
    assert!(lanes(&[], 30).is_empty());
}
