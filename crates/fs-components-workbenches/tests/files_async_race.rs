//! What the person opens while a long call runs is not thrown away when its answer comes in.
//!
//! Tests: State tests over a source in memory. The answer of an upload or download goes to the line and the transfer
//! row; a rename being typed, a move on its way or a question already open stay as they are.

#[path = "files_common/mod.rs"]
mod files_common;

use std::sync::Arc;

use files_common::{Shelf, settle};
use fs_components_workbenches::file_manager::{
    Ask, CarryKind, FileLabels, FilesState, Notice, Variant,
};

/// An upload is on its way; `broken` makes the server fail it.
fn uploading(broken: bool) -> (Arc<Shelf>, FilesState) {
    let shelf = Arc::new(Shelf::new().file("a").file("b").folder("d", None));
    let mut s = FilesState::new(shelf.clone(), FileLabels::default(), Variant::Full);
    s.begin_upload();
    s.set_input("/tmp/new.txt");
    s.submit_input();
    assert!(s.is_running());
    if broken {
        *shelf.broken.lock().unwrap() = Some("down".into());
    }
    (shelf, s)
}

fn typed_rename(s: &mut FilesState) {
    s.select("a");
    s.begin_rename();
    s.set_input("typed-by-person");
}

fn rename_kept(s: &FilesState) -> bool {
    matches!(s.ask(), Ask::Input(i) if i.value == "typed-by-person")
}

#[test]
fn an_upload_that_went_leaves_a_rename_being_typed() {
    let (_shelf, mut s) = uploading(false);
    typed_rename(&mut s);
    settle(&mut s);
    assert!(rename_kept(&s));
}

#[test]
fn an_upload_that_failed_leaves_a_rename_being_typed_and_says_so_on_the_line() {
    let (_shelf, mut s) = uploading(true);
    typed_rename(&mut s);
    settle(&mut s);
    assert!(rename_kept(&s));
    assert!(matches!(s.notice(), Some(Notice::Problem(..))));
}

#[test]
fn a_taken_name_does_not_replace_what_is_open() {
    let (shelf, mut s) = uploading(false);
    shelf.change("a", |t| t.name = "new.txt".into());
    s.select("b");
    s.begin_rename();
    s.set_input("typed-by-person");
    settle(&mut s);
    assert!(matches!(s.ask(), Ask::Input(i) if i.value == "typed-by-person"));
}

#[test]
fn an_upload_that_went_leaves_a_move_on_its_way() {
    let (_shelf, mut s) = uploading(false);
    s.select("a");
    s.carry(CarryKind::Move);
    assert!(s.carrying().is_some());
    settle(&mut s);
    assert!(s.carrying().is_some());
}

#[test]
fn a_download_that_failed_leaves_a_rename_being_typed() {
    let shelf = Arc::new(Shelf::new().file("a").file("b"));
    let mut s = FilesState::new(shelf.clone(), FileLabels::default(), Variant::Full);
    s.select("a");
    s.begin_download();
    s.set_input("/tmp/out.txt");
    s.submit_input();
    assert!(s.is_running());
    *shelf.broken.lock().unwrap() = Some("down".into());
    s.select("b");
    s.begin_rename();
    s.set_input("typed-by-person");
    settle(&mut s);
    assert!(rename_kept(&s));
}

#[test]
fn a_rescan_from_the_bar_leaves_a_rename_being_typed() {
    let shelf = Arc::new(Shelf::new().file("a"));
    let mut s = FilesState::new(shelf, FileLabels::default(), Variant::Full);
    s.select("a");
    s.begin_rename();
    s.set_input("typed-by-person");
    s.rescan();
    assert!(s.is_running());
    assert!(rename_kept(&s), "the rescan closed the input");
    settle(&mut s);
    assert!(rename_kept(&s));
}
