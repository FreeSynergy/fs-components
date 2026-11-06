//! A long status line: cut with an ellipsis, shown whole on a press or Enter. Through the real router,
//! hit test and overlay stack; no window is opened. A hang here is a deadlock, not slowness.

#[path = "menu_common/mod.rs"]
mod menu_common;

use fs_components_app::Status;
use menu_common::fixture::{FILE, harness, open_menu};
use menu_common::{ENTER, ESCAPE, Harness};

const LINE: &str = "fs-app-status";
const PANEL: &str = "fs-app-status-panel";

fn long() -> Status {
    let start = "Imported 3, 0 already known. ".repeat(12);
    Status::Problem(format!("{start}failed: the last words"))
}

fn with_long() -> Harness {
    let (mut h, _, _) = harness(&["a"]);
    h.app.set_status(Some(long()));
    h.rebuild();
    h.frame();
    h
}

#[test]
fn a_press_on_a_long_line_shows_it_whole_inside_the_window_and_a_second_closes_it() {
    let mut h = with_long();
    h.click(LINE);
    assert!(h.has(PANEL));
    let (x, y, w, height) = h.bounds(PANEL);
    assert!(x >= 0.0 && x + w <= 800.0, "{x} {w}");
    assert!(
        y >= 0.0 && y + height <= 500.0 - 28.0 + 1.0,
        "above the line: {y} {height}"
    );
    assert!(height > 28.0, "the text wraps: {height}");
    h.click(LINE);
    h.settle();
    assert!(!h.has(PANEL));
}

#[test]
fn a_short_line_opens_nothing() {
    let (mut h, _, _) = harness(&["a"]);
    h.app.set_status(Some(Status::Info("Saved.".into())));
    h.rebuild();
    h.frame();
    h.click(LINE);
    assert!(!h.has(PANEL));
    h.app.set_status(None);
    h.frame();
    h.click(LINE);
    assert!(!h.has(PANEL));
}

#[test]
fn a_press_outside_closes_it_also_after_a_rebuild() {
    let mut h = with_long();
    h.click(LINE);
    assert!(h.has(PANEL));
    h.rebuild();
    h.frame();
    assert!(h.has(PANEL), "a rebuild does not close it");
    h.press_at((400.0, 200.0));
    h.settle();
    assert!(!h.has(PANEL));
    h.click(LINE);
    assert!(h.has(PANEL), "and it opens again");
}

#[test]
fn enter_opens_it_from_the_keyboard_and_escape_closes_it() {
    let mut h = with_long();
    for _ in 0..12 {
        h.key(menu_common::TAB);
        if h.focused(LINE) {
            break;
        }
    }
    assert!(h.focused(LINE), "a tab stop");
    h.key(ENTER);
    h.frame();
    assert!(h.has(PANEL));
    h.key(ESCAPE);
    h.settle();
    assert!(!h.has(PANEL));
    h.key(ENTER);
    h.frame();
    assert!(h.has(PANEL), "still known to be closed");
}

#[test]
fn a_new_message_closes_the_panel_with_the_old_one() {
    let mut h = with_long();
    h.click(LINE);
    assert!(h.has(PANEL));
    h.app.set_status(Some(Status::Info("Done.".into())));
    h.settle();
    assert!(!h.has(PANEL));
}

#[test]
fn a_menu_and_the_panel_close_each_other() {
    let mut h = with_long();
    h.click(LINE);
    h.click(FILE);
    h.settle();
    assert!(!h.has(PANEL));
    assert_eq!(open_menu(&h), Some(0));
    h.click(LINE);
    h.settle();
    assert_eq!(open_menu(&h), None);
    assert!(h.has(PANEL));
}

#[test]
fn a_text_taller_than_the_window_scrolls_to_its_last_words_in_the_panel() {
    let (mut h, _, _) = harness(&["a"]);
    h.app.set_status(Some(Status::Problem(
        "One failure after another. ".repeat(600),
    )));
    h.rebuild();
    h.frame();
    h.click(LINE);
    assert!(h.has(PANEL));
    let (_, y, _, height) = h.bounds(PANEL);
    assert!(
        y >= 0.0 && height <= 500.0,
        "inside the window: {y} {height}"
    );
    let (_, top, _, _) = h.bounds("fs-app-status-text");
    let (x, _, w, _) = h.bounds(PANEL);
    let moved = h.wheel_at((x + w / 2.0, y + height / 2.0), 300.0);
    let (_, after, _, _) = h.bounds("fs-app-status-text");
    assert!(after < top - 100.0, "it scrolled: {top} {after} {moved}");
}
