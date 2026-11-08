//! Editing, creating and deleting: the same rules as the reader's editing.
//!
//! Tests: State machine tests, no window.

#[path = "people_common/mod.rs"]
mod people_common;

use std::sync::Arc;

use fs_components_workbenches::people::{Outcome, PeopleLabels, PeopleState, Shown, Variant};
use people_common::Book;

fn state(names: &[&str]) -> (Arc<Book>, PeopleState) {
    let book = Arc::new(Book::with(names));
    let state = PeopleState::new(book.clone(), PeopleLabels::default(), Variant::Full);
    (book, state)
}

#[test]
fn a_save_writes_and_shows_the_person_again() {
    let (book, mut s) = state(&["ada", "bob"]);
    s.begin_edit();
    s.set_field("name", "Ada L.");
    s.save();
    assert_eq!(book.name("ada").as_deref(), Some("Ada L."));
    assert!(s.editing().is_none());
    assert!(matches!(s.shown(), Shown::Person { sheet, .. } if sheet.title == "Ada L."));
    assert_eq!(s.notice(), Some("Saved."));
    assert_eq!(s.rows()[0].cells[0], "Ada L.", "the list follows");
}

#[test]
fn a_save_without_changes_writes_nothing() {
    let (book, mut s) = state(&["ada"]);
    s.begin_edit();
    s.set_field("name", "other");
    s.set_field("name", "ada");
    s.save();
    assert_eq!(book.writes(), 0);
    assert_eq!(s.notice(), Some("Nothing was changed."));
    assert!(s.editing().is_none());
}

#[test]
fn unsaved_changes_block_leaving_and_new_but_not_the_search() {
    let (_, mut s) = state(&["ada", "bob"]);
    s.begin_edit();
    s.set_field("name", "changed");
    assert_eq!(s.select("bob"), Outcome::Blocked);
    assert_eq!(s.begin_create(), Outcome::Blocked);
    assert_eq!(s.notice(), Some("Save or cancel your changes first."));
    assert_eq!(s.selected(), Some("ada"));
    assert_eq!(
        s.editing().unwrap().draft.fields().next().unwrap().value,
        "changed"
    );
    s.search("b");
    assert_eq!(s.rows().len(), 1);
    assert_eq!(
        s.editing().unwrap().draft.fields().next().unwrap().value,
        "changed"
    );
}

#[test]
fn an_untouched_edit_does_not_block() {
    let (_, mut s) = state(&["ada", "bob"]);
    s.begin_edit();
    assert_eq!(s.select("bob"), Outcome::Done);
    assert!(s.editing().is_none());
    assert_eq!(s.selected(), Some("bob"));
}

#[test]
fn cancel_with_changes_only_asks_and_discarding_is_another_action() {
    let (book, mut s) = state(&["ada"]);
    s.begin_edit();
    s.set_field("name", "changed");
    s.cancel_edit();
    assert!(s.editing().is_some(), "only asked");
    assert_eq!(
        s.notice(),
        Some("You have unsaved changes. Keep editing, or discard them.")
    );
    s.cancel_edit();
    assert!(s.editing().is_some(), "a double-click only asks");
    s.set_field("name", "changed more");
    s.confirm_discard();
    assert!(s.editing().is_some(), "typing took the question back");
    s.cancel_edit();
    s.keep_editing();
    s.confirm_discard();
    assert!(s.editing().is_some(), "answered: keep editing");
    s.cancel_edit();
    s.confirm_discard();
    assert!(s.editing().is_none());
    assert_eq!(book.name("ada").as_deref(), Some("ada"));
}

#[test]
fn a_refused_save_keeps_the_draft_and_shows_why() {
    let (book, mut s) = state(&["ada"]);
    s.begin_edit();
    s.set_field("name", "changed");
    *book.refuse.lock().unwrap() = Some("needs an encrypted store".into());
    s.save();
    let edit = s.editing().unwrap();
    assert!(
        edit.error
            .as_deref()
            .unwrap()
            .contains("needs an encrypted store")
    );
    assert_eq!(edit.draft.fields().next().unwrap().value, "changed");
    *book.refuse.lock().unwrap() = None;
    s.save();
    assert_eq!(book.name("ada").as_deref(), Some("changed"));
}

#[test]
fn a_conflict_changes_nothing_says_so_and_the_second_save_writes_knowingly() {
    let (book, mut s) = state(&["ada"]);
    s.begin_edit();
    s.set_field("name", "mine");
    book.other_changes("ada", "theirs");
    s.save();
    assert_eq!(
        book.name("ada").as_deref(),
        Some("theirs"),
        "nothing written"
    );
    let edit = s.editing().unwrap();
    assert!(edit.error.as_deref().unwrap().contains("Someone else"));
    assert_eq!(edit.draft.fields().next().unwrap().value, "mine");
    s.save();
    assert_eq!(book.name("ada").as_deref(), Some("mine"));
}

#[test]
fn a_new_person_is_made_listed_and_selected() {
    let (book, mut s) = state(&["ada"]);
    assert_eq!(s.begin_create(), Outcome::Done);
    assert!(s.editing().unwrap().key.is_none());
    s.set_field("name", "zoe");
    s.save();
    assert_eq!(book.name("zoe").as_deref(), Some("zoe"));
    assert_eq!(s.selected(), Some("zoe"));
    assert_eq!(s.rows().len(), 2);
    assert_eq!(s.notice(), Some("Created."));
}

#[test]
fn a_new_person_left_empty_writes_nothing_and_the_old_one_stays_shown() {
    let (book, mut s) = state(&["ada"]);
    s.begin_create();
    s.save();
    assert_eq!(book.writes(), 0);
    assert_eq!(s.selected(), Some("ada"));
}

#[test]
fn fields_can_be_added_where_the_group_allows_it() {
    let (_, mut s) = state(&["ada"]);
    s.begin_edit();
    s.add_field("mail");
    s.add_field("name");
    let edit = s.editing().unwrap();
    let keys: Vec<_> = edit.draft.fields().map(|f| f.key.as_str()).collect();
    assert_eq!(keys, ["name", "mail/1", "mail/+2"]);
    assert!(!edit.draft.changed(), "an empty new field is no change");
    let added = edit.draft.fields().last().unwrap().key.clone();
    s.set_field(&added, "second@x.org");
    assert!(s.editing().unwrap().draft.changed());
}

#[test]
fn deleting_takes_a_second_action_for_the_same_person() {
    let (book, mut s) = state(&["ada", "bob", "cid"]);
    s.ask_delete();
    assert!(s.delete_asked());
    assert!(s.notice().unwrap().contains("ada"));
    s.ask_delete();
    assert!(book.name("ada").is_some(), "asking twice is not confirming");

    s.select("bob");
    assert!(!s.delete_asked(), "another person drops the question");
    s.confirm_delete();
    assert!(book.name("bob").is_some(), "not asked for bob yet");
    s.ask_delete();
    s.keep();
    assert!(!s.delete_asked());
    s.ask_delete();
    s.confirm_delete();
    assert!(book.name("bob").is_none());
    assert_eq!(book.name("ada").as_deref(), Some("ada"));
    assert_eq!(s.selected(), Some("cid"), "the next person moves up");
    assert_eq!(s.notice(), Some("bob was deleted."));
}

#[test]
fn deleting_the_last_person_leaves_nothing_shown() {
    let (_, mut s) = state(&["ada"]);
    s.ask_delete();
    s.confirm_delete();
    assert_eq!(s.shown(), &Shown::Nothing);
    assert!(s.rows().is_empty());
}

#[test]
fn a_refused_delete_keeps_the_person_and_shows_why() {
    let (book, mut s) = state(&["ada"]);
    *book.refuse.lock().unwrap() = Some("it is locked".into());
    s.ask_delete();
    s.confirm_delete();
    assert!(s.notice().unwrap().contains("it is locked"));
    assert_eq!(s.selected(), Some("ada"));
    assert!(!s.delete_asked());
}

#[test]
fn nothing_can_be_deleted_during_an_edit() {
    let (book, mut s) = state(&["ada"]);
    s.begin_edit();
    assert!(!s.can_delete());
    s.ask_delete();
    s.confirm_delete();
    assert!(book.name("ada").is_some());
}

#[test]
fn after_a_refused_save_cancel_reads_the_person_again() {
    let (book, mut s) = state(&["ada"]);
    s.begin_edit();
    s.set_field("name", "mine");
    book.other_changes("ada", "theirs");
    *book.refuse.lock().unwrap() = Some("no".into());
    s.save();
    s.cancel_edit();
    s.confirm_discard();
    assert!(matches!(s.shown(), Shown::Person { sheet, .. } if sheet.title == "theirs"));
    assert_eq!(s.rows()[0].cells[0], "theirs", "the row too");
}

#[test]
fn a_person_someone_else_deleted_leaves_on_cancel_and_on_delete() {
    let (book, mut s) = state(&["ada", "bob"]);
    s.begin_edit();
    book.other_deletes("ada");
    s.set_field("name", "mine");
    s.save();
    assert!(s.editing().is_some(), "the draft stays while they read why");
    s.cancel_edit();
    s.confirm_discard();
    assert_eq!(s.shown(), &Shown::Nothing);
    assert_eq!(s.notice(), Some("This person no longer exists."));
    assert_eq!(s.rows().len(), 1);

    let (book, mut s) = state(&["ada", "bob"]);
    s.ask_delete();
    book.other_deletes("ada");
    s.confirm_delete();
    assert_eq!(s.rows().len(), 1, "gone from the list, no error for ever");
    assert_eq!(s.notice(), Some("This person no longer exists."));
    assert_eq!(s.shown(), &Shown::Nothing);
}

#[test]
fn save_with_no_changes_reads_the_person_and_the_row_again() {
    let (book, mut s) = state(&["ada"]);
    s.begin_edit();
    book.other_changes("ada", "theirs");
    s.save();
    assert_eq!(s.notice(), Some("Nothing was changed."));
    assert!(matches!(s.shown(), Shown::Person { sheet, .. } if sheet.title == "theirs"));
    assert_eq!(s.rows()[0].cells[0], "theirs");
    assert_eq!(book.writes(), 0);
}

#[test]
fn a_delete_that_went_through_with_a_remark_says_both() {
    let (book, mut s) = state(&["ada", "bob"]);
    *book.delete_remark.lock().unwrap() = Some("Their links remain.".into());
    s.ask_delete();
    s.confirm_delete();
    assert_eq!(s.notice(), Some("ada was deleted. Their links remain."));
    assert_eq!(s.rows().len(), 1);
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
