//! The state machine without a window.
//!
//! Tests: State tests over a source in memory. What only the real core can show is in `fs-files`.

#[path = "files_common/mod.rs"]
mod files_common;

use std::sync::Arc;

use files_common::Shelf;
use fs_components_workbenches::file_manager::{
    Ask, CarryKind, FileLabels, FileView, FilesState, Variant,
};

fn state(shelf: Shelf) -> (Arc<Shelf>, FilesState) {
    let shelf = Arc::new(shelf);
    let state = FilesState::new(shelf.clone(), FileLabels::default(), Variant::Full);
    (shelf, state)
}

fn notice(state: &FilesState) -> String {
    state
        .notice()
        .map(|n| n.text().to_string())
        .unwrap_or_default()
}

#[test]
fn nothing_is_selected_or_read_at_the_start_and_the_context_is_a_plain_value() {
    let (_, s) = state(Shelf::new().file("a"));
    let context = s.context();
    assert_eq!(context.selected, None);
    assert_eq!(context.folder, None);
    assert!(!context.in_trash);
    assert_eq!(context.view, FileView::List);
}

#[test]
fn a_taken_name_asks_and_changes_nothing_and_back_keeps_what_was_typed() {
    let (shelf, mut s) = state(Shelf::new().file("a").file("b"));
    s.select("a");
    s.begin_rename();
    s.set_input("b.txt");
    s.submit_input();
    assert!(matches!(s.ask(), Ask::NameTaken(_)));
    assert_eq!(shelf.thing("a").unwrap().name, "a.txt");
    s.name_taken_back();
    match s.ask() {
        Ask::Input(input) => assert_eq!(input.value, "b.txt"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_name_the_backend_cannot_hold_is_said_as_such_and_the_input_stays() {
    let (shelf, mut s) = state(Shelf::new().file("a"));
    s.select("a");
    s.begin_rename();
    s.set_input("../x");
    s.submit_input();
    match s.ask() {
        Ask::Input(input) => {
            assert_eq!(input.value, "../x");
            assert!(input.error.as_deref().unwrap().contains("cannot be used"));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(shelf.writes(), 0);
}

#[test]
fn a_failed_move_says_why_and_changes_nothing_and_a_thing_gone_leaves_the_view() {
    let (shelf, mut s) = state(Shelf::new().folder("d", None).file("a").file("b"));
    s.select("a");
    s.carry(CarryKind::Move);
    s.open_folder(Some("d"));
    assert!(s.carrying().is_some());
    shelf.break_it(Some("disk full"));
    s.put_down();
    assert!(notice(&s).contains("disk full"));
    shelf.break_it(None);
    assert_eq!(shelf.thing("a").unwrap().folder, None);
    s.open_folder(None);
    s.select("b");
    shelf.things.lock().unwrap().remove("b");
    s.retry();
    s.select("a");
    s.select("a");
    s.begin_rename();
    shelf.things.lock().unwrap().remove("a");
    s.set_input("c.txt");
    s.submit_input();
    assert!(s.rows().iter().all(|r| r.key != "a"), "gone from the list");
    assert_eq!(s.selected_key(), None);
}

#[test]
fn a_failed_read_keeps_the_rows_and_a_retry_comes_back() {
    let (shelf, mut s) = state(Shelf::new().numbered(10));
    assert_eq!(s.rows().len(), 10);
    shelf.break_it(Some("the store is gone"));
    s.retry();
    assert_eq!(s.rows().len(), 10, "the rows stay");
    assert!(s.list_problem().unwrap().contains("the store is gone"));
    shelf.break_it(None);
    s.retry();
    assert_eq!(s.list_problem(), None);
}

#[test]
fn one_bad_entry_is_named_and_the_rest_stays() {
    let shelf = Shelf::new().numbered(3);
    shelf.unreadable.lock().unwrap().push("broken.bin".into());
    let (_, s) = state(shelf);
    assert_eq!(s.rows().len(), 3);
    assert_eq!(s.unreadable(), ["broken.bin"]);
}

#[test]
fn pages_of_the_source_are_put_together_and_more_keeps_on_after_a_failure() {
    let (shelf, mut s) = state(Shelf::new().numbered(450));
    assert_eq!(s.rows().len(), 200);
    assert!(s.has_more());
    shelf.break_it(Some("gone"));
    s.load_more();
    assert_eq!(s.rows().len(), 200);
    assert!(s.list_problem().is_some());
    assert!(s.has_more(), "it can be tried again");
    shelf.break_it(None);
    s.load_more();
    assert_eq!(s.rows().len(), 400);
    s.load_more();
    assert_eq!(s.rows().len(), 450);
    assert!(!s.has_more());
}
