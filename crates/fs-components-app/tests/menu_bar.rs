//! When the title bar has menus, and what it does with what it is given. No window is opened.
//!
//! Tests: Input tests. A hang here is a deadlock, not slowness.

#[path = "menu_common/mod.rs"]
mod menu_common;

use std::sync::Arc;

use fs_components_app::{AppWidget, Menu, MenuEntry, Screen};
use menu_common::fixture::{FILE, Target, VIEW, app_with, entry, harness, open_menu};
use menu_common::{Harness, TAB};

#[test]
fn without_menus_there_is_no_bar_of_menus_and_tab_goes_to_the_navigation() {
    let presses = Arc::default();
    let screens = vec![
        Screen::new("a", "a", Arc::new(Target(Arc::clone(&presses)))),
        Screen::new("b", "b", Arc::new(Target(presses))),
    ];
    let mut h = Harness::new(AppWidget::new("T", screens).unwrap(), 800.0, 500.0);
    assert!(!h.has("fs-app-menus"));
    h.key(TAB);
    assert!(h.router.focused().is_some(), "a navigation entry");
}

#[test]
fn a_menu_without_entries_is_not_drawn() {
    let (mut h, _, _) = harness(&["a"]);
    h.app
        .set_menus(vec![Menu::new("Empty", Vec::new())], |_| {})
        .unwrap();
    h.rebuild();
    assert!(!h.has(FILE));
    assert!(!h.has("fs-app-menus"));
}

#[test]
fn new_menus_reach_the_bar_through_an_old_element() {
    let (mut h, log, _) = harness(&["a"]);
    h.click(FILE);
    let chosen = Arc::clone(&log);
    let changed = vec![
        Menu::new("File", vec![MenuEntry::new("first", "First").disabled()]),
        Menu::new("View", vec![MenuEntry::new("show", "Show")]),
    ];
    h.app
        .set_menus(changed, move |id| {
            chosen.lock().unwrap().push(id.to_owned())
        })
        .unwrap();
    assert_eq!(open_menu(&h), None, "new menus close the open one");
    h.rebuild();
    h.settle();
    h.click(FILE);
    assert!(!h.has(&entry("third")), "the new lines");
    h.click(&entry("first"));
    assert!(log.lock().unwrap().is_empty(), "now disabled");
}

#[test]
fn a_long_title_gives_way_to_the_menus() {
    let presses = Arc::default();
    let (app, _) = app_with(&"A very long title ".repeat(30), &["a"], &presses);
    let h = Harness::new(app, 800.0, 500.0);
    let (x, _, w, _) = h.bounds(VIEW);
    assert!(x + w <= 800.0, "the last menu ends at {}", x + w);
}
