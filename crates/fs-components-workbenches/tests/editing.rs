//! Editing and exporting through the document source.
//!
//! Pattern: State machine tests — begin, type, save, cancel, export, and what they refuse.

mod common;

use std::sync::Arc;

use common::{Editable, Fake};
use fs_components_workbenches::{Labels, Reader, Shown};

fn reader() -> (Arc<Editable>, Reader) {
    let source = Arc::new(Editable::with(&[("a", "first\n"), ("b", "second")]));
    let mut r = Reader::new(source.clone(), Labels::default());
    r.set_export_target(Some("out".into()));
    (source, r)
}

#[test]
fn a_read_only_source_offers_no_edit_and_refuses() {
    let mut r = Reader::new(Arc::new(Fake), Labels::default());
    assert!(!r.can_edit() && !r.can_export());
    r.begin_edit();
    assert!(r.editing().is_none());
    assert!(r.notice().unwrap().contains("cannot be edited"));
}

#[test]
fn edit_and_save_changes_the_source_and_the_shown_document() {
    let (source, mut r) = reader();
    r.begin_edit();
    assert_eq!(r.editing().unwrap().draft, "first\n");
    r.set_draft("changed");
    assert!(r.editing().unwrap().changed());
    r.save();
    assert!(r.editing().is_none());
    assert_eq!(
        source.text("a"),
        "changed\n",
        "the final newline of the original is kept"
    );
    assert!(
        source.exported.lock().unwrap().is_empty(),
        "saving must not export"
    );
    assert_eq!(r.notice(), Some("Saved."));
    match r.shown() {
        Shown::Document { document, .. } => {
            assert_eq!(document.blocks[0].plain_text(), "changed\n");
            assert_eq!(document.revision.as_deref(), Some("2"));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn saving_without_changes_writes_nothing() {
    let (source, mut r) = reader();
    r.begin_edit();
    source.other_saves("a", "theirs\n");
    r.save();
    assert_eq!(
        source.text("a"),
        "theirs\n",
        "no write, so no conflict and no loss"
    );
    assert_eq!(r.notice(), Some("Nothing was changed."));
}

#[test]
fn another_persons_save_is_a_conflict_and_the_draft_stays() {
    let (source, mut r) = reader();
    r.begin_edit();
    r.set_draft("mine");
    source.other_saves("a", "theirs\n");
    r.save();
    assert_eq!(source.text("a"), "theirs\n", "their work must survive");
    let e = r.editing().expect("still editing");
    assert_eq!(e.draft, "mine\n");
    assert!(e.error.as_ref().unwrap().contains("Someone else"));
}

#[test]
fn the_text_area_changes_of_line_end_do_not_count_as_changes() {
    let source = Arc::new(Editable::with(&[("a", "one\r\ntwo\r\n")]));
    let mut r = Reader::new(source, Labels::default());
    r.begin_edit();
    r.set_draft("one\ntwo");
    assert!(!r.editing().unwrap().changed());
    r.set_draft("one\ntwo\nthree");
    assert_eq!(r.editing().unwrap().draft, "one\r\ntwo\r\nthree\r\n");
}

#[test]
fn a_refused_save_keeps_the_draft_and_says_why() {
    let (source, mut r) = reader();
    r.begin_edit();
    r.set_draft("FORBIDDEN");
    r.save();
    let e = r.editing().unwrap();
    assert_eq!(e.draft, "FORBIDDEN\n");
    assert!(e.error.as_ref().unwrap().contains("refused"));
    assert_eq!(source.text("a"), "first\n");
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
    assert_eq!(
        r.follow("anything"),
        fs_components_workbenches::Outcome::Blocked
    );
    r.cancel_edit();
    assert!(r.editing().is_some(), "the first cancel only asks");
    r.cancel_edit();
    assert!(r.editing().is_none());
    r.open("a");
    assert_eq!(r.current_path(), Some("a"));
}

#[test]
fn begin_edit_during_an_edit_keeps_the_draft() {
    let (_s, mut r) = reader();
    r.begin_edit();
    r.set_draft("typed");
    r.begin_edit();
    assert_eq!(r.editing().unwrap().draft, "typed\n");
}

#[test]
fn a_save_refreshes_search_results() {
    let (_s, mut r) = reader();
    r.search("needle");
    assert!(r.hits().is_empty());
    r.begin_edit();
    r.set_draft("a needle");
    r.save();
    assert_eq!(r.hits().len(), 1);
}

#[test]
fn export_is_its_own_action_and_asks_before_replacing() {
    let (source, mut r) = reader();
    r.export();
    assert_eq!(r.notice(), Some("Exported a to out."));
    r.export();
    assert!(
        r.notice().unwrap().contains("Press Export again"),
        "{:?}",
        r.notice()
    );
    r.export();
    assert!(source.exported.lock().unwrap().last().unwrap().2);
}

#[test]
fn per_path_permission_decides_the_edit_button() {
    let source = Arc::new(Editable::with(&[("locked", "x"), ("free", "y")]));
    let mut r = Reader::new(source, Labels::default());
    r.open("locked");
    assert!(!r.can_edit());
    r.open("free");
    assert!(r.can_edit());
}

#[test]
fn text_the_area_cannot_hold_is_kept_exactly_or_announced() {
    let source = Arc::new(Editable::with(&[
        ("blank", "x\n\n"),
        ("mixed", "a\r\nb\nc"),
        ("crlf", "a\r\nb\r\n"),
        ("lf", "a\nb\n"),
    ]));
    let mut r = Reader::new(source.clone(), Labels::default());
    // Untouched: whatever the text area gives back, the original is what stays.
    for (path, typed) in [
        ("blank", "x"),
        ("mixed", "a\nb\nc"),
        ("crlf", "a\nb"),
        ("lf", "a\nb"),
    ] {
        r.open(path);
        r.begin_edit();
        r.set_draft(typed);
        assert!(!r.editing().unwrap().changed(), "{path} counts as changed");
        r.cancel_edit();
    }
    // The person is told what saving would change.
    r.open("blank");
    r.begin_edit();
    assert!(
        r.editing()
            .unwrap()
            .note
            .as_ref()
            .unwrap()
            .contains("blank lines")
    );
    r.cancel_edit();
    r.open("mixed");
    r.begin_edit();
    assert!(
        r.editing()
            .unwrap()
            .note
            .as_ref()
            .unwrap()
            .contains("line ends")
    );
    r.cancel_edit();
    r.open("lf");
    r.begin_edit();
    assert!(r.editing().unwrap().note.is_none());
    // A paste of CRLF on LF leaves no stray \r; a paste of LF on CRLF stays CRLF.
    r.set_draft("a\r\nb\r\nc");
    assert_eq!(r.editing().unwrap().draft, "a\nb\nc\n");
    r.cancel_edit();
    r.cancel_edit();
    r.open("crlf");
    r.begin_edit();
    r.set_draft("a\nb\nc");
    assert_eq!(r.editing().unwrap().draft, "a\r\nb\r\nc\r\n");
}

#[test]
fn the_overwrite_confirmation_belongs_to_path_and_target() {
    let (source, mut r) = reader();
    r.export();
    r.export();
    assert!(r.notice().unwrap().contains("Press Export again"));
    // The target changes: the next press asks again instead of replacing.
    r.set_export_target(Some("elsewhere".into()));
    r.export();
    assert!(source.exported.lock().unwrap().iter().all(|e| !e.2));
    // Edit and cancel clear a pending confirmation.
    r.set_export_target(Some("out".into()));
    r.export();
    assert!(r.notice().unwrap().contains("Press Export again"));
    r.begin_edit();
    r.cancel_edit();
    r.export();
    assert!(
        r.notice().unwrap().contains("Press Export again"),
        "must ask again"
    );
}

#[test]
fn after_a_conflict_the_next_save_replaces_knowingly() {
    let (source, mut r) = reader();
    r.begin_edit();
    r.set_draft("mine");
    source.other_saves("a", "theirs\n");
    r.save();
    assert!(
        r.editing()
            .unwrap()
            .error
            .as_ref()
            .unwrap()
            .contains("Press Save again")
    );
    assert_eq!(source.text("a"), "theirs\n");
    r.save();
    assert!(r.editing().is_none());
    assert_eq!(source.text("a"), "mine\n");
}

#[test]
fn a_failed_list_stays_visible_after_search_and_save() {
    use common::Failing;
    let mut r = Reader::new(Arc::new(Failing), Labels::default());
    assert!(r.problem().unwrap().contains("list"));
    r.search("x");
    r.search("");
    assert!(
        r.problem().unwrap().contains("list"),
        "an empty search must not erase it"
    );
}
