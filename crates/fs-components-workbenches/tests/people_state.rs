//! The state machine of the people workbench: list, selection, search, more, compact.
//!
//! Pattern: State machine tests, no window.

#[path = "people_common/mod.rs"]
mod people_common;

use std::sync::Arc;

use fs_components_workbenches::people::{
    Outcome, PAGE, PeopleCapabilities, PeopleLabels, PeopleState, Shown, Variant,
};
use people_common::Book;

fn state(book: Book) -> (Arc<Book>, PeopleState) {
    let book = Arc::new(book);
    let state = PeopleState::new(book.clone(), PeopleLabels::default(), Variant::Full);
    (book, state)
}

#[test]
fn it_opens_with_the_first_person_shown() {
    let (_, s) = state(Book::with(&["ada", "bob"]));
    assert_eq!(s.rows().len(), 2);
    assert_eq!(s.selected(), Some("ada"));
    assert!(!s.has_more());
}

#[test]
fn an_empty_source_shows_nothing_and_offers_new_only() {
    let (_, s) = state(Book::with(&[]));
    assert_eq!(s.shown(), &Shown::Nothing);
    assert!(s.can_create());
    assert!(!s.can_edit() && !s.can_delete());
}

#[test]
fn rows_come_a_page_at_a_time_and_more_brings_the_rest() {
    let (_, mut s) = state(Book::numbered(PAGE + 5));
    assert_eq!(s.rows().len(), PAGE);
    assert!(s.has_more());
    s.load_more();
    assert_eq!(s.rows().len(), PAGE + 5);
    assert!(!s.has_more());
    assert_eq!(s.rows()[PAGE + 4].key, format!("p{:05}", PAGE + 4));
}

#[test]
fn exactly_one_page_has_no_more() {
    let (_, s) = state(Book::numbered(PAGE));
    assert_eq!(s.rows().len(), PAGE);
    assert!(!s.has_more());
}

#[test]
fn a_search_starts_a_new_list_and_keeps_the_person_shown() {
    let (_, mut s) = state(Book::with(&["ada", "bob", "bert"]));
    let generation = s.list_generation();
    s.search("  b ");
    assert_eq!(s.term(), "b");
    assert_eq!(s.rows().len(), 2);
    assert_eq!(s.selected(), Some("ada"));
    assert_ne!(s.list_generation(), generation);
    s.search("b");
    assert_eq!(s.term(), "b");
    s.search("");
    assert_eq!(s.rows().len(), 3);
}

#[test]
fn a_failing_list_is_kept_and_shown_until_it_works() {
    let (book, mut s) = state(Book::with(&["ada"]));
    *book.broken_list.lock().unwrap() = Some("the disk is gone".into());
    s.search("a");
    assert!(s.list_problem().unwrap().contains("the disk is gone"));
    assert!(s.rows().is_empty());
    assert_eq!(s.selected(), Some("ada"), "the person stays");
    *book.broken_list.lock().unwrap() = None;
    s.search("ad");
    assert_eq!(s.list_problem(), None);
    assert_eq!(s.rows().len(), 1);
}

#[test]
fn a_person_who_cannot_be_read_says_why_and_offers_no_edit() {
    let (_, mut s) = state(Book::with(&["ada"]));
    assert_eq!(s.select("broken"), Outcome::Done);
    assert!(matches!(s.shown(), Shown::Failed { reason, .. } if reason.contains("damaged")));
    assert!(!s.can_edit());
}

#[test]
fn what_the_source_cannot_do_is_not_offered_and_not_done() {
    let mut book = Book::with(&["ada"]);
    book.capabilities = PeopleCapabilities::read_only();
    let (book, mut s) = state(book);
    assert!(!s.can_edit() && !s.can_create() && !s.can_delete());
    s.begin_edit();
    assert!(s.editing().is_none());
    assert_eq!(s.begin_create(), Outcome::Done);
    assert!(s.editing().is_none());
    s.ask_delete();
    s.confirm_delete();
    assert_eq!(book.name("ada").as_deref(), Some("ada"));
}

#[test]
fn the_compact_variant_only_reads_whatever_the_source_can_do() {
    let book = Arc::new(Book::with(&["ada", "bob"]));
    let mut s = PeopleState::new(book.clone(), PeopleLabels::default(), Variant::Compact);
    assert!(!s.can_edit() && !s.can_create() && !s.can_delete());
    assert_eq!(s.select("bob"), Outcome::Done);
    s.begin_edit();
    assert!(s.editing().is_none());
    assert_eq!(s.selected(), Some("bob"));
    assert_eq!(book.writes(), 0);
}
