//! The reader's transitions, without a window.
//!
//! Pattern: State machine tests — every transition on its own.

mod common;

use std::sync::Arc;

use common::Fake;
use fs_components_workbenches::{Labels, Outcome, Reader, Shown};

fn reader() -> Reader {
    Reader::new(Arc::new(Fake), Labels::default())
}

#[test]
fn starts_at_the_sources_start_document() {
    let r = reader();
    assert_eq!(r.current_path(), Some("dir/b"));
    assert!(!r.can_go_back());
    assert_eq!(r.entries().len(), 3);
    assert_eq!(r.problem(), None);
}

#[test]
fn open_and_back() {
    let mut r = reader();
    r.open("a");
    assert_eq!(r.current_path(), Some("a"));
    assert!(r.can_go_back());
    r.back();
    assert_eq!(r.current_path(), Some("dir/b"));
    assert!(!r.can_go_back());
    r.back();
    assert_eq!(r.current_path(), Some("dir/b"));
}

#[test]
fn opening_the_document_shown_does_not_grow_the_history() {
    let mut r = reader();
    r.open("dir/b");
    assert!(!r.can_go_back());
}

#[test]
fn follow_document_external_and_missing() {
    let mut r = reader();
    assert_eq!(r.follow("up"), Outcome::Opened);
    assert_eq!(r.current_path(), Some("a"));
    assert_eq!(
        r.follow("web"),
        Outcome::External("https://example.org".into())
    );
    assert_eq!(r.current_path(), Some("a"));
    assert_eq!(r.follow("nowhere"), Outcome::Missing);
    assert!(r.notice().unwrap().contains("a -> nowhere"));
    r.open("dir/b");
    assert_eq!(r.notice(), None);
}

#[test]
fn a_document_that_cannot_be_read_is_shown_with_its_reason() {
    let mut r = reader();
    r.open("broken");
    match r.shown() {
        Shown::Failed { path, reason } => {
            assert_eq!(path, "broken");
            assert!(reason.contains("the file is damaged"), "{reason}");
        }
        other => panic!("{other:?}"),
    }
    r.back();
    assert!(matches!(r.shown(), Shown::Document { .. }));
}

#[test]
fn search_and_clear() {
    let mut r = reader();
    r.search("  dir ");
    assert_eq!(r.term(), "dir");
    assert_eq!(r.hits().len(), 1);
    assert_eq!(r.hits()[0].path, "dir/b");
    r.search("");
    assert!(r.hits().is_empty());
}

#[test]
fn a_failed_search_is_reported_not_hidden() {
    let mut r = reader();
    r.search("boom");
    assert!(r.hits().is_empty());
    assert!(r.problem().unwrap().contains("the index is gone"));
    r.search("dir");
    assert_eq!(r.problem(), None);
}

#[test]
fn all_texts_come_from_the_caller() {
    let mut labels = Labels::default();
    labels.missing_target = "Nichts bei \"{}\".".into();
    let mut r = Reader::new(Arc::new(Fake), labels);
    r.follow("x");
    assert!(r.notice().unwrap().starts_with("Nichts bei"));
}

#[test]
fn characters_no_font_has_are_announced_once() {
    use fs_components_workbenches::{
        Block, Document, DocumentSource, Entry, Hit, Inline, LinkTarget, SourceError,
    };
    struct Odd;
    impl DocumentSource for Odd {
        fn entries(&self, _: &str) -> Result<Vec<Entry>, SourceError> {
            Ok(vec![Entry::new("a", "A"), Entry::new("b", "B")])
        }
        fn read(&self, _: &str) -> Result<Document, SourceError> {
            // U+1FAFF is in the emoji block and unassigned: no font has a glyph for it.
            Ok(Document::new(
                "d",
                vec![Block::Paragraph(Inline::plain("odd \u{1FAFF} sign"))],
            ))
        }
        fn resolve(&self, _: &str, h: &str) -> LinkTarget {
            LinkTarget::Missing(h.into())
        }
        fn find(&self, _: &str) -> Result<Vec<Hit>, SourceError> {
            Ok(vec![])
        }
    }
    if fs_components_widgets::fallback_family('\u{1FAFF}').is_some() {
        eprintln!("skipped: this machine has a font for the test character");
        return;
    }
    let mut r = Reader::new(Arc::new(Odd), Labels::default());
    assert!(r.notice().unwrap().contains("U+1FAFF"), "{:?}", r.notice());
    r.open("b");
    assert_eq!(r.notice(), None, "the same character was announced twice");
}
