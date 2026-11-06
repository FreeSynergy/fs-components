//! The menus of the real shell, pressed through Blinc's router, hit test and overlay stack. No window
//! is opened.
//!
//! Pattern: Input tests. A hang here is a deadlock, not slowness.

#[path = "menu_common/mod.rs"]
mod menu_common;

use std::sync::atomic::Ordering;

use fs_components_app::Status;
use menu_common::fixture::{FILE, PANEL, VIEW, entry, harness, menus, open_menu};

#[test]
fn a_press_opens_the_menu_under_its_label_and_a_second_closes_it() {
    let (mut h, log, _) = harness(&["a"]);
    assert!(!h.has(PANEL));
    h.click(FILE);
    assert_eq!(open_menu(&h), Some(0));
    let (px, py, _, _) = h.bounds(PANEL);
    let (tx, ty, _, th) = h.bounds(FILE);
    assert!((px - tx).abs() < 1.0, "under its label: {px} {tx}");
    assert!(py >= ty + th, "below its label: {py}");
    assert!(h.has(&entry("first")) && h.has(&entry("off")));
    h.click(FILE);
    assert_eq!(open_menu(&h), None);
    h.settle();
    assert!(!h.has(PANEL));
    assert!(log.lock().unwrap().is_empty());
}

#[test]
fn a_press_on_a_line_over_the_surface_reaches_the_handler_and_not_the_surface() {
    let (mut h, log, presses) = harness(&["a"]);
    h.click(FILE);
    let point = h.centre(&entry("third"));
    h.press_at(point);
    h.frame();
    assert_eq!(*log.lock().unwrap(), ["third"]);
    assert_eq!(presses.load(Ordering::SeqCst), 0, "not passed through");
    assert_eq!(open_menu(&h), None);
    h.settle();
    assert!(!h.has(PANEL));
    h.press_at(point);
    assert_eq!(presses.load(Ordering::SeqCst), 1, "the surface was below");
}

/// Long enough for the navigation's buttons to reach under a panel.
const NAV: [&str; 4] = [
    "The first long screen name",
    "The second long screen name",
    "The third long screen name",
    "The fourth long screen name",
];

#[test]
fn a_press_on_a_line_over_the_navigation_does_not_change_the_screen() {
    let (mut h, log, _) = harness(&[NAV[0], NAV[1], NAV[2], NAV[3]]);
    h.click(FILE);
    let (px, _, _, _) = h.bounds(PANEL);
    let point = (px + 10.0, h.centre(&entry("third")).1);
    assert!(point.0 < 200.0, "the panel lies over the navigation");
    h.press_at(point);
    h.frame();
    assert_eq!(*log.lock().unwrap(), ["third"]);
    h.app
        .with_shell(|shell| assert_eq!(shell.current().unwrap().id, NAV[0]));
    h.settle();
    h.press_at(point);
    h.app.with_shell(|shell| {
        assert_ne!(
            shell.current().unwrap().id,
            NAV[0],
            "a navigation entry was below"
        );
    });
}

#[test]
fn a_disabled_line_does_nothing_and_the_menu_stays_open() {
    let (mut h, log, presses) = harness(&["a"]);
    h.click(FILE);
    h.click(&entry("off"));
    assert!(log.lock().unwrap().is_empty());
    assert_eq!(presses.load(Ordering::SeqCst), 0);
    assert_eq!(open_menu(&h), Some(0));
    assert!(h.has(PANEL));
}

#[test]
fn a_double_click_on_a_line_chooses_once() {
    let (mut h, log, _) = harness(&["a"]);
    h.click(FILE);
    let point = h.centre(&entry("first"));
    h.press_at(point);
    h.press_at(point);
    h.frame();
    h.press_at(point);
    assert_eq!(*log.lock().unwrap(), ["first"]);
}

#[test]
fn a_press_outside_closes_the_menu_and_a_rebuild_does_not_lose_that() {
    for rebuilt in [false, true] {
        let (mut h, log, presses) = harness(&["a"]);
        h.click(FILE);
        if rebuilt {
            h.rebuild();
            assert_eq!(open_menu(&h), Some(0), "a rebuild leaves it open");
            assert!(h.has(PANEL));
        }
        h.press_at((700.0, 400.0));
        h.frame();
        assert_eq!(open_menu(&h), None, "rebuilt: {rebuilt}");
        h.settle();
        assert!(!h.has(PANEL));
        assert!(log.lock().unwrap().is_empty());
        assert_eq!(
            presses.load(Ordering::SeqCst),
            1,
            "the press was the surface's"
        );
    }
}

#[test]
fn a_press_on_the_other_menu_switches() {
    let (mut h, _, _) = harness(&["a"]);
    h.click(FILE);
    h.click(VIEW);
    assert_eq!(open_menu(&h), Some(1));
    h.settle();
    assert!(h.has(&entry("show")));
    assert!(!h.has(&entry("first")));
    h.click(&entry("show"));
    assert_eq!(open_menu(&h), None);
}

#[test]
fn the_handler_changes_screen_and_status_from_a_real_press() {
    let (mut h, _, _) = harness(&["a", "b"]);
    let this = h.app.clone();
    h.app
        .set_menus(menus(), move |id| {
            this.select("b");
            this.set_status(Some(Status::Problem(id.to_owned())));
        })
        .unwrap();
    h.rebuild();
    h.click(FILE);
    h.click(&entry("first"));
    h.rebuild();
    h.app.with_shell(|shell| {
        assert_eq!(shell.current().unwrap().id, "b");
        assert_eq!(shell.status(), Some(&Status::Problem("first".into())));
    });
    h.click(FILE);
    assert_eq!(
        open_menu(&h),
        Some(0),
        "the bar still works after the rebuild"
    );
}
