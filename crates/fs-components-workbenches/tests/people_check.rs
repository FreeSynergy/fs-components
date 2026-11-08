//! A field the source judges: named as typed, and an invalid one never reaches the source.
//!
//! Tests: State machine tests, no window.

#[path = "people_common/mod.rs"]
mod people_common;

use std::sync::Arc;

use fs_components_workbenches::people::{PeopleLabels, PeopleState, Variant};
use people_common::Book;

fn state(names: &[&str]) -> (Arc<Book>, PeopleState) {
    let book = Arc::new(Book::with(names));
    let state = PeopleState::new(book.clone(), PeopleLabels::default(), Variant::Full);
    (book, state)
}

#[test]
fn an_invalid_address_is_named_as_typed_and_blocks_the_save() {
    let (book, mut s) = state(&["ada"]);
    s.begin_edit();
    assert_eq!(s.problem("mail/1"), None, "what is stored is not judged");
    assert!(s.typed("mail/1", "ada"), "valid to invalid: draw again");
    assert_eq!(
        s.problem("mail/1").as_deref(),
        Some("an address needs an @")
    );
    assert!(!s.typed("mail/1", "ada."), "same problem: no redraw");
    s.save();
    assert_eq!(book.writes(), 0, "nothing written");
    assert!(
        s.editing()
            .unwrap()
            .error
            .as_deref()
            .unwrap()
            .contains("not valid")
    );
    assert!(
        s.typed("mail/1", "ada@y.org"),
        "invalid to valid: draw again"
    );
    s.save();
    assert!(s.editing().is_none());
    assert_eq!(book.writes(), 1);
    // Emptied means "remove this value": no problem.
    s.begin_edit();
    assert!(!s.typed("mail/1", ""));
}
