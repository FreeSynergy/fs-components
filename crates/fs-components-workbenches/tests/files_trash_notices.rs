//! What is said after the trash was emptied, or found empty.
//!
//! Tests: State tests over a source in memory.

#[path = "files_common/mod.rs"]
mod files_common;

use std::sync::Arc;

use files_common::Shelf;
use fs_components_workbenches::file_manager::{FileLabels, FilesState, Variant};

fn state(shelf: Shelf) -> (Arc<Shelf>, FilesState) {
    let shelf = Arc::new(shelf);
    let state = FilesState::new(shelf.clone(), FileLabels::default(), Variant::Full);
    (shelf, state)
}

fn said(s: &FilesState) -> String {
    s.notice().map(|n| n.text().to_string()).unwrap_or_default()
}

#[test]
fn an_empty_trash_is_said_even_when_the_file_selected_has_just_gone() {
    let (shelf, mut s) = state(Shelf::new().file("a"));
    shelf.change("a", |t| t.trashed = true);
    s.open_trash();
    s.select("a");
    shelf.things.lock().unwrap().remove("a");
    s.ask_empty_trash();
    assert_eq!(said(&s), FileLabels::default().trash_empty);
}

#[test]
fn what_could_not_be_deleted_is_one_sentence_of_its_own_with_the_names() {
    let (shelf, mut s) = state(Shelf::new().file("a").file("b"));
    for key in ["a", "b"] {
        shelf.change(key, |t| t.trashed = true);
    }
    shelf.stays.lock().unwrap().push("b".into());
    s.open_trash();
    s.ask_empty_trash();
    s.really();
    let text = said(&s);
    assert_eq!(
        text,
        FileLabels::fill(&FileLabels::default().emptied_failed, "b.txt")
    );
    assert!(!text.contains("{}"), "{text}");
}
