//! What is typed is not thrown away by another act, and a source that cannot change anything is not
//! asked to (the picker changes nothing).
//!
//! Pattern: State tests over a source in memory, and the buttons of the bar through the harness.

#[path = "files_common/mod.rs"]
mod files_common;

use std::sync::Arc;

use files_common::harness::Harness;
use files_common::{Shelf, widget};
use fs_components_workbenches::file_manager::{
    Ask, CarryKind, FileLabels, FilePicker, FilesState, Variant,
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
