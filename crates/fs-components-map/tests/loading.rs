//! Fetching tiles: a hanging source gives up its place, what is no longer wanted gives up its place, a failure is not
//! kept for good, a panic leaves the tile empty, and the oldest tiles go when there are too many.
//!
//! Pattern: none (tests). The sources are made to answer, hang, fail or panic on demand.

mod common;
mod loading_support;

use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::time::Duration;

use common::serial;
use fs_components_app::Surface;
use fs_components_map::{MOST_LOADING, MapPosition, TileId};
use loading_support::*;

#[test]
fn a_source_that_panics_leaves_the_tile_empty_and_the_map_alive() {
    let _g = serial();
    let source = Scripted::new(Mode::Panic);
    let s = surface(&source, limits(5000, 60_000, 256), 0.0);
    let _ = s.build(100.0, 100.0);
    until("the worker asked", || source.asked() > 0);
    std::thread::sleep(Duration::from_millis(100));
    let before = source.asked();
    let _ = s.build(100.0, 100.0);
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(s.tiles_kept(), 0, "no tile, an empty one");
    assert_eq!(
        source.asked(),
        before,
        "a failed tile is not asked for in the next draw"
    );
}

#[test]
fn at_most_four_tiles_are_fetched_at_once_and_a_hanging_source_gives_up_its_places_after_the_time()
{
    let _g = serial();
    let source = Scripted::new(Mode::Hang);
    let s = surface(&source, limits(150, 60_000, 256), 5.0);
    let _ = s.build(900.0, 700.0);
    until("four asked", || source.asked() >= MOST_LOADING);
    std::thread::sleep(Duration::from_millis(60));
    assert_eq!(source.asked(), MOST_LOADING, "no fifth while four hang");
    // after the time the four places are free: the next draw asks for others (bounded by the workers alive)
    std::thread::sleep(Duration::from_millis(200));
    draw_until(&s, "the places came back", || source.asked() > MOST_LOADING);
    *source.release.lock().unwrap() = true;
    until("all workers come back", || {
        source.running.load(Ordering::SeqCst) == 0
    });
}

#[test]
fn a_tile_that_is_no_longer_wanted_gives_up_its_place() {
    let _g = serial();
    let source = Scripted::new(Mode::Hang);
    let s = surface(&source, limits(60_000, 60_000, 256), 5.0);
    let _ = s.build(900.0, 700.0);
    until("four asked", || source.asked() >= MOST_LOADING);
    let first: HashSet<TileId> = source.asked.lock().unwrap().iter().copied().collect();
    // the view goes far away: the four places are free at once, without waiting for the time limit
    s.go_to(MapPosition::new(-120.0, -33.0, 5.0));
    draw_until(&s, "the new view asked", || {
        source
            .asked
            .lock()
            .unwrap()
            .iter()
            .any(|t| !first.contains(t))
    });
    *source.release.lock().unwrap() = true;
}

#[test]
fn a_failure_is_not_kept_for_good_it_is_asked_again_after_the_time() {
    let _g = serial();
    let source = Scripted::new(Mode::Fail);
    let s = surface(&source, limits(5000, 700, 256), 0.0);
    let _ = s.build(100.0, 100.0);
    until("asked", || source.asked() > 0);
    let once = source.asked();
    std::thread::sleep(Duration::from_millis(60));
    let _ = s.build(100.0, 100.0);
    assert_eq!(source.asked(), once, "not again at once");
    source.set(Mode::Answer);
    std::thread::sleep(Duration::from_millis(750));
    draw_until(&s, "the tile arrived after the retry", || {
        s.tiles_kept() > 0
    });
}

#[test]
fn no_such_tile_is_kept_for_good() {
    let _g = serial();
    let source = Scripted::new(Mode::Missing);
    let s = surface(&source, limits(5000, 50, 256), 0.0);
    draw_until(&s, "kept", || s.tiles_kept() > 0);
    let asked = source.asked();
    std::thread::sleep(Duration::from_millis(250));
    let _ = s.build(100.0, 100.0);
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(
        source.asked(),
        asked,
        "an answer of 'none' is not asked again"
    );
}

#[test]
fn the_time_limit_acts_without_a_draw_a_watchdog_frees_the_places_on_time() {
    let _g = serial();
    let source = Scripted::new(Mode::Hang);
    let s = surface(&source, limits(120, 60_000, 256), 5.0);
    let _ = s.build(900.0, 700.0);
    until("four hold places", || s.tiles_loading() == MOST_LOADING);
    // no draw from here on: the places come back by themselves
    until("the places came back with no draw", || {
        s.tiles_loading() == 0
    });
    *source.release.lock().unwrap() = true;
}

#[test]
fn a_source_that_never_answers_stalls_the_map_and_says_so_until_it_comes_back() {
    let _g = serial();
    let source = Scripted::new(Mode::Hang);
    let s = surface(&source, limits(40, 60_000, 256), 5.0);
    assert!(!s.source_stalled());
    draw_until(&s, "eight workers hang", || s.source_stalled());
    // `in_flight` rises before the worker calls `tile()`: wait until all eight have asked, then look
    until("all eight asked", || {
        source.asked() >= fs_components_map::MOST_IN_FLIGHT
    });
    std::thread::sleep(Duration::from_millis(100));
    let _ = s.build(900.0, 700.0);
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(
        source.asked(),
        fs_components_map::MOST_IN_FLIGHT,
        "no ninth thread"
    );
    *source.release.lock().unwrap() = true;
    until("they come back", || !s.source_stalled());
}
