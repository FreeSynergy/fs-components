//! Editing and exporting through the data source.
//!
//! Pattern: State machine tests — begin, type, save, cancel, export, and what they refuse.

mod common;

use std::sync::Arc;

use common::{Editable, Fake};
use fs_components_workbenches::{Reader, Shown};

fn reader() -> (Arc<Editable>, Reader) {
    let source = Arc::new(Editable::with(&[("a", "first"), ("b", "second")]));
    (source.clone(), Reader::new(source))
}

#[test]
fn a_read_only_source_offers_no_edit_and_refuses() {
    let mut r = Reader::new(Arc::new(Fake));
    assert!(!r.can_edit() && !r.can_export());
    r.begin_edit();
    assert!(r.editing().is_none());
    assert!(r.notice().unwrap().contains("cannot be edited"));
}

#[test]
fn edit_and_save_changes_the_source_and_the_page() {
    let (source, mut r) = reader();
    r.begin_edit();
    assert_eq!(r.editing().unwrap().draft, "first");
    r.set_draft("changed");
    assert!(r.editing().unwrap().changed());
    r.save();
    assert!(r.editing().is_none());
    assert_eq!(source.texts.lock().unwrap()["a"], "changed");
    assert!(
        source.exported.lock().unwrap().is_empty(),
        "saving must not export"
    );
    assert_eq!(r.notice(), Some("Saved."));
    match r.shown() {
        Shown::Page { document, .. } => assert_eq!(document.blocks[0].plain_text(), "changed"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_refused_save_keeps_the_draft_and_says_why() {
    let (source, mut r) = reader();
    r.begin_edit();
    r.set_draft("FORBIDDEN");
    r.save();
    let e = r.editing().unwrap();
    assert_eq!(e.draft, "FORBIDDEN");
    assert!(e.error.as_ref().unwrap().contains("refused"));
    assert_eq!(source.texts.lock().unwrap()["a"], "first");
}

#[test]
fn unsaved_changes_block_leaving_but_untouched_edits_do_not() {
    let (_s, mut r) = reader();
    r.begin_edit();
    r.open("b");
    assert_eq!(r.current_path(), Some("b"), "an untouched edit lets you go");
    r.begin_edit();
    r.set_draft("x");
    r.open("a");
    assert_eq!(r.current_path(), Some("b"));
    assert!(r.notice().unwrap().contains("Save or cancel"));
    r.cancel_edit();
    r.open("a");
    assert_eq!(r.current_path(), Some("a"));
}

#[test]
fn export_is_its_own_action() {
    let (source, mut r) = reader();
    r.export();
    assert_eq!(*source.exported.lock().unwrap(), ["a"]);
    assert_eq!(r.notice(), Some("Exported a."));
}
