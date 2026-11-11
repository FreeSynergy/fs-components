//! The tiles that are kept: the oldest go when there are too many, and when they take too many bytes.
//!
//! Pattern: none (tests). The source answers on demand.

mod common;
mod loading_support;

use std::time::Duration;

use common::serial;
use fs_components_map::{Limits, MapPosition};
use loading_support::*;

#[test]
fn the_oldest_tiles_go_when_there_are_too_many() {
    let _g = serial();
    let source = Scripted::new(Mode::Answer);
    let s = surface(&source, limits(5000, 60_000, 3), 4.0);
    // a view of many tiles, then others elsewhere: never more than three are kept
    for lon in [0.0, 40.0, 80.0, 120.0] {
        s.go_to(MapPosition::new(lon, 30.0, 4.0));
        let wanted = source.asked() + 1;
        draw_until(&s, "something new arrived", || source.asked() >= wanted);
        std::thread::sleep(Duration::from_millis(50));
        assert!(s.tiles_kept() <= 3, "{} kept", s.tiles_kept());
    }
    assert!(source.asked() > 3, "more were fetched than are kept");
    assert!(s.tiles_kept() > 0);
}

#[test]
fn the_kept_tiles_are_bounded_in_bytes_not_only_in_number() {
    let _g = serial();
    let source = Scripted::new(Mode::Answer);
    let one = fs_components_map::decode_tile(&source.bytes)
        .expect("the fixture decodes")
        .memory();
    assert!(one > 0);
    let bound = 2 * one + one / 2; // room for two tiles, not for three
    let s = surface(
        &source,
        Limits {
            most_bytes: bound,
            ..limits(5000, 60_000, 256)
        },
        4.0,
    );
    for lon in [0.0, 40.0, 80.0, 120.0, 160.0] {
        s.go_to(MapPosition::new(lon, 30.0, 4.0));
        let wanted = source.asked() + 1;
        draw_until(&s, "something new arrived", || source.asked() >= wanted);
        std::thread::sleep(Duration::from_millis(50));
        assert!(
            s.tiles_kept_bytes() <= bound,
            "{} bytes kept, bound {bound}",
            s.tiles_kept_bytes()
        );
    }
    assert!(s.tiles_kept() >= 1 && s.tiles_kept_bytes() > 0);
}
