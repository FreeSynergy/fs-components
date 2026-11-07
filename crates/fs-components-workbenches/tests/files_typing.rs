//! What is typed is not thrown away by another act, and a source that cannot change anything is not
//! asked to (the picker changes nothing).
//!
//! Tests: State tests over a source in memory, and the buttons of the bar through the harness.

#[path = "files_common/mod.rs"]
mod files_common;

use std::sync::Arc;

use files_common::harness::Harness;
use files_common::{Shelf, widget};
use fs_components_workbenches::file_manager::{
    Ask, CarryKind, FileLabels, FilePicker, FilesState, PendingUpload, Variant,
};

fn typing_state() -> (Arc<Shelf>, FilesState) {
    let shelf = Arc::new(Shelf::new().folder("d", None).file("a"));
    let mut s = FilesState::new(shelf.clone(), FileLabels::default(), Variant::Full);
    s.select("a");
    s.begin_rename();
    s.set_input("typed name");
    (shelf, s)
}

fn typed(s: &FilesState) -> Option<String> {
    match s.ask() {
        Ask::Input(input) => Some(input.value.clone()),
        _ => None,
    }
}

#[test]
fn delete_and_carry_leave_a_name_being_typed_alone() {
    let (_, mut s) = typing_state();
    s.ask_delete();
    assert_eq!(typed(&s).as_deref(), Some("typed name"), "Delete");
    for kind in [CarryKind::Move, CarryKind::Copy] {
        s.carry(kind);
        assert_eq!(typed(&s).as_deref(), Some("typed name"), "{kind:?}");
        assert!(!s.can_put_down());
    }
}

#[test]
fn the_bar_does_not_offer_what_would_throw_the_typing_away() {
    let (_, w) = widget(Shelf::new().folder("d", None).file("a"));
    w.with_state(|s| s.select("a"));
    let mut h = Harness::new(w, 1300.0, 700.0);
    assert!(h.has("delete") && h.has("move"));
    h.widget.with_state(|s| {
        s.begin_rename();
        s.set_input("typed name");
    });
    h.rebuild();
    assert!(!h.has("delete") && !h.has("move") && !h.has("copy"));
}

#[test]
fn a_picker_carries_and_puts_down_nothing() {
    let shelf = Arc::new(Shelf::new().folder("d", None).file("a"));
    let picker = FilePicker::new(shelf.clone(), FileLabels::default());
    picker.with_state(|s| {
        s.select("a");
        s.carry(CarryKind::Move);
        s.open_folder(Some("d"));
        assert!(!s.can_put_down());
        s.put_down();
    });
    assert_eq!(shelf.writes(), 0);
    assert_eq!(shelf.thing("a").unwrap().folder, None);
}

#[test]
fn the_transfers_do_not_offer_a_retry_while_a_name_is_typed() {
    let shelf = Shelf::new().file("a");
    shelf
        .pending
        .lock()
        .unwrap()
        .push(PendingUpload::new("u1", "big.iso", 500));
    let (_, w) = widget(shelf);
    w.with_state(FilesState::open_transfers);
    let mut h = Harness::new(w, 1300.0, 700.0);
    let key = h.widget.with_state(|s| s.transfers()[0].shown.key.clone());
    let retry = format!("fs-tl-{key}-retry");
    assert!(h.has_id(&retry), "the retry is there when nothing is typed");
    h.widget.with_state(|s| {
        s.begin_new_folder();
        s.set_input("typed name");
    });
    h.rebuild();
    assert!(!h.has_id(&retry), "a retry that does nothing is offered");
}

#[test]
fn restore_and_empty_trash_leave_a_name_being_typed_alone() {
    let shelf = Arc::new(Shelf::new().file("a"));
    shelf.change("a", |t| t.trashed = true);
    let mut s = FilesState::new(shelf.clone(), FileLabels::default(), Variant::Full);
    s.begin_new_folder();
    s.set_input("typed name");
    s.open_trash();
    s.select("a");
    s.restore_selected();
    assert!(shelf.thing("a").unwrap().trashed, "restored while typing");
    s.ask_empty_trash();
    assert_eq!(typed(&s).as_deref(), Some("typed name"), "Empty trash");
}

#[test]
fn the_question_about_a_taken_name_still_guards_what_was_typed() {
    let shelf = Arc::new(Shelf::new().file("a").file("b"));
    let mut s = FilesState::new(shelf, FileLabels::default(), Variant::Full);
    s.select("a");
    s.begin_rename();
    s.set_input("b.txt");
    s.submit_input();
    assert!(matches!(s.ask(), Ask::NameTaken(_)), "a name taken asks");
    s.ask_delete();
    s.carry(CarryKind::Move);
    assert!(matches!(s.ask(), Ask::NameTaken(_)), "the typing was thrown away");
}

#[test]
fn a_picker_can_start_no_input_so_nothing_typed_stands_in_the_way_of_choosing() {
    // `can_choose` also asks `typing()`; a picker reads only, so it cannot be true there
    let picker = FilePicker::new(Arc::new(Shelf::new().file("a")), FileLabels::default());
    picker.with_state(|s| {
        s.select("a");
        s.begin_new_folder();
        s.begin_rename();
        s.begin_upload();
        s.begin_download();
        assert!(matches!(s.ask(), Ask::Nothing), "an input was started");
        assert!(s.can_choose());
        s.choose();
    });
    assert_eq!(picker.picked().unwrap().key, "a");
}
