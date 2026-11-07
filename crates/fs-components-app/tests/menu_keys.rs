//! The menus of the real shell, reached and chosen with the keyboard the way a window passes keys to
//! Blinc's navigation. No window is opened.
//!
//! Tests: Input tests. A hang here is a deadlock, not slowness.

#[path = "menu_common/mod.rs"]
mod menu_common;

use fs_components_app::{Menu, MenuEntry};
use menu_common::fixture::{FILE, PANEL, VIEW, entry, harness, open_menu};
use menu_common::{DOWN, ENTER, ESCAPE, LEFT, RIGHT, TAB, UP};

#[test]
fn the_keyboard_reaches_opens_walks_and_chooses() {
    let (mut h, log, _) = harness(&["a"]);
    h.key(TAB);
    assert!(h.focused(FILE), "Tab lands on the first menu");
    h.key(RIGHT);
    assert!(h.focused(VIEW));
    h.key(LEFT);
    assert!(h.focused(FILE));
    h.key(DOWN);
    assert_eq!(open_menu(&h), Some(0));
    h.frame();
    assert!(h.focused(&entry("first")), "Down opens on the first line");
    h.key(DOWN);
    assert!(h.focused(&entry("third")), "the disabled line is skipped");
    h.key(ENTER);
    assert_eq!(*log.lock().unwrap(), ["third"]);
    assert_eq!(open_menu(&h), None);
    h.settle();
    assert!(h.focused(FILE), "the keyboard goes back to the menu");
}

#[test]
fn up_opens_on_the_last_line_and_escape_goes_back() {
    let (mut h, log, _) = harness(&["a"]);
    h.key(TAB);
    h.key(UP);
    h.frame();
    assert!(h.focused(&entry("fourth")));
    h.key(ESCAPE);
    h.settle();
    assert_eq!(open_menu(&h), None);
    assert!(!h.has(PANEL));
    assert!(h.focused(FILE));
    assert!(log.lock().unwrap().is_empty());
}

#[test]
fn left_and_right_in_an_open_menu_switch_to_the_next_menu() {
    let (mut h, _, _) = harness(&["a"]);
    h.key(TAB);
    h.key(DOWN);
    h.frame();
    h.key(RIGHT);
    h.frame();
    assert_eq!(open_menu(&h), Some(1));
    h.settle();
    assert!(h.focused(&entry("show")), "the keyboard moved along");
    h.key(ESCAPE);
    h.settle();
    assert_eq!(open_menu(&h), None);
    assert!(h.focused(VIEW));
}

#[test]
fn a_menu_of_only_disabled_lines_opens_and_escape_closes_it() {
    let (mut h, log, _) = harness(&["a"]);
    let only = vec![Menu::new(
        "Off",
        vec![MenuEntry::new("off", "Off").disabled()],
    )];
    h.app.set_menus(only, |_| {}).unwrap();
    h.rebuild();
    h.key(TAB);
    assert!(h.focused(FILE));
    h.key(DOWN);
    assert_eq!(open_menu(&h), Some(0));
    h.key(DOWN);
    h.key(ENTER);
    assert_eq!(open_menu(&h), Some(0), "nothing to choose");
    h.key(ESCAPE);
    h.settle();
    assert_eq!(open_menu(&h), None);
    assert!(log.lock().unwrap().is_empty());
}

#[test]
fn right_in_the_only_menu_closes_it() {
    let (mut h, _, _) = harness(&["a"]);
    let one = vec![Menu::new("One", vec![MenuEntry::new("x", "X")])];
    h.app.set_menus(one, |_| {}).unwrap();
    h.rebuild();
    h.key(TAB);
    h.key(DOWN);
    h.frame();
    h.key(RIGHT);
    h.settle();
    assert_eq!(open_menu(&h), None);
    assert!(h.focused(FILE));
}
