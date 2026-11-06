//! The reader's transitions, without a window.
//!
//! Pattern: State machine tests — every transition on its own.

mod common;

use std::sync::Arc;

use common::Fake;
use fs_components_workbenches::{Outcome, Reader, Shown};

fn reader() -> Reader {
    Reader::new(Arc::new(Fake))
}

#[test]
fn starts_at_the_sources_start_page() {
    let r = reader();
    assert_eq!(r.current_path(), Some("dir/b"));
    assert!(!r.can_go_back());
    assert_eq!(r.entries().len(), 3);
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
fn opening_the_page_shown_does_not_grow_the_history() {
    let mut r = reader();
    r.open("dir/b");
    assert!(!r.can_go_back());
}

#[test]
fn follow_page_external_and_missing() {
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
fn a_page_that_cannot_be_read_is_shown_with_its_reason() {
    let mut r = reader();
    r.open("broken");
    match r.shown() {
        Shown::Failed { path, reason } => {
            assert_eq!(path, "broken");
            assert_eq!(reason, "the file is damaged");
        }
        other => panic!("{other:?}"),
    }
    r.back();
    assert!(matches!(r.shown(), Shown::Page { .. }));
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
