//! The state machine of the calendar workbench: editing and deleting.
//!
//! Pattern: State machine tests, no window.

#[path = "calendar_common/mod.rs"]
mod calendar_common;

use std::sync::Arc;

use calendar_common::Diary;
use fs_components_workbenches::calendar::{CalendarLabels, CalendarState, Outcome, Shown, Variant};

fn state(diary: Diary) -> (Arc<Diary>, CalendarState) {
    let diary = Arc::new(diary);
    let state = CalendarState::new(diary.clone(), CalendarLabels::default(), Variant::Full);
    (diary, state)
}

fn pair() -> Diary {
    Diary::with(&[("ada", "2026-11-04"), ("bob", "2026-11-20")])
}

#[test]
fn unsaved_changes_block_leaving_but_not_moving_the_calendar() {
    let (_, mut s) = state(pair());
    s.select("ada");
    s.begin_edit();
    s.set_field("title", "Ada L.");
    assert_eq!(s.select("bob"), Outcome::Blocked);
    assert_eq!(s.close(), Outcome::Blocked);
    assert_eq!(s.begin_create(), Outcome::Blocked);
    assert!(s.notice().unwrap().contains("Save or cancel"));
    assert_eq!(s.selected(), Some("ada"));
    s.next();
    assert_eq!(s.editing().unwrap().draft.value("title"), Some("Ada L."));
}

#[test]
fn a_locked_field_cannot_be_typed_in_and_a_flag_is_pressed() {
    let (_, mut s) = state(pair());
    s.select("ada");
    s.begin_edit();
    s.set_field("fixed", "changed");
    assert_eq!(s.editing().unwrap().draft.value("fixed"), Some("kept"));
    s.toggle_field("all_day");
    assert_eq!(s.editing().unwrap().draft.value("all_day"), Some("true"));
    s.toggle_field("title");
    assert_eq!(
        s.editing().unwrap().draft.value("title"),
        Some("ada"),
        "not a flag"
    );
    s.toggle_field("all_day");
    assert!(!s.editing().unwrap().draft.changed());
}

#[test]
fn save_without_changes_writes_nothing() {
    let (diary, mut s) = state(pair());
    s.select("ada");
    s.begin_edit();
    s.save();
    assert_eq!(diary.writes(), 0);
    assert_eq!(s.notice(), Some("Nothing was changed."));
    assert_eq!(s.selected(), Some("ada"));
}

#[test]
fn a_failed_save_keeps_the_draft_and_cancel_reads_again() {
    let (diary, mut s) = state(pair());
    s.select("ada");
    s.begin_edit();
    s.set_field("title", "Mine");
    *diary.refuse.lock().unwrap() = Some("the store refuses".into());
    s.save();
    let edit = s.editing().unwrap();
    assert_eq!(edit.draft.value("title"), Some("Mine"));
    assert!(edit.error.as_deref().unwrap().contains("the store refuses"));
    diary.other_changes("ada", "Theirs");
    s.cancel_edit();
    assert!(s.editing().is_some(), "the first Cancel only asks");
    s.cancel_edit();
    assert!(s.editing().is_none());
    assert!(matches!(s.shown(), Shown::Item { sheet, .. } if sheet.title == "Theirs"));
    assert!(
        s.items().iter().any(|i| i.title == "Theirs"),
        "the view read again"
    );
}

#[test]
fn something_gone_from_the_store_leaves_the_view_after_cancel_and_after_a_save() {
    let (diary, mut s) = state(pair());
    s.select("ada");
    s.begin_edit();
    s.set_field("title", "Mine");
    diary.other_deletes("ada");
    s.cancel_edit();
    s.cancel_edit();
    assert_eq!(s.shown(), &Shown::Nothing);
    assert!(s.notice().unwrap().contains("no longer exists"));
    assert!(s.items().iter().all(|i| i.key != "ada"));

    s.select("bob");
    s.begin_edit();
    s.set_field("title", "Mine");
    diary.other_deletes("bob");
    s.save();
    assert!(s.editing().is_none());
    assert_eq!(s.shown(), &Shown::Nothing);
    assert!(s.items().iter().all(|i| i.key != "bob"));
}

#[test]
fn a_conflict_keeps_the_draft_and_the_next_save_writes_knowingly() {
    let (diary, mut s) = state(pair());
    s.select("ada");
    s.begin_edit();
    s.set_field("title", "Mine");
    diary.other_changes("ada", "Theirs");
    s.save();
    assert_eq!(diary.title("ada").as_deref(), Some("Theirs"));
    assert!(
        s.editing()
            .unwrap()
            .error
            .as_deref()
            .unwrap()
            .contains("Press Save again")
    );
    s.save();
    assert_eq!(diary.title("ada").as_deref(), Some("Mine"));
}

#[test]
fn an_unreadable_item_says_so_and_a_failed_read_of_the_items_too() {
    let (diary, mut s) = state(pair());
    s.select("broken");
    assert!(matches!(s.shown(), Shown::Failed { reason, .. } if reason.contains("damaged")));
    assert!(!s.can_edit());
    *diary.broken.lock().unwrap() = Some("no store".into());
    s.next();
    assert!(s.problem().unwrap().contains("no store"));
    assert!(s.items().is_empty());
    *diary.broken.lock().unwrap() = None;
    s.next();
    assert_eq!(s.problem(), None);
}

#[test]
fn asking_and_confirming_are_two_actions_and_asking_twice_deletes_nothing() {
    let (diary, mut s) = state(pair());
    s.select("ada");
    s.confirm_delete();
    assert!(diary.title("ada").is_some(), "not asked yet");
    s.ask_delete();
    s.ask_delete();
    assert!(diary.title("ada").is_some());
    assert!(s.delete_asked());
    s.keep();
    assert!(!s.delete_asked());
    s.confirm_delete();
    assert!(diary.title("ada").is_some(), "the answer was no");
    s.ask_delete();
    s.confirm_delete();
    assert!(diary.title("ada").is_none());
    assert_eq!(s.shown(), &Shown::Nothing, "nothing else opens");
    assert_eq!(s.notice(), Some("ada was deleted."));
}

#[test]
fn the_question_is_tied_to_the_item_and_says_what_goes_with_it() {
    let (diary, mut s) = state(Diary::with(&[
        ("series", "2026-11-04"),
        ("bob", "2026-11-20"),
    ]));
    s.select("series");
    s.ask_delete();
    assert!(s.notice().unwrap().contains("All of the series goes."));
    s.select("bob");
    assert!(!s.delete_asked());
    s.confirm_delete();
    assert!(diary.title("bob").is_some() && diary.title("series").is_some());
}

#[test]
fn a_failed_delete_keeps_the_item_and_one_gone_leaves() {
    let (diary, mut s) = state(pair());
    s.select("ada");
    s.ask_delete();
    *diary.refuse.lock().unwrap() = Some("no".into());
    s.confirm_delete();
    assert!(s.notice().unwrap().contains("Not deleted"));
    assert_eq!(s.selected(), Some("ada"));
    *diary.refuse.lock().unwrap() = None;
    s.ask_delete();
    diary.other_deletes("ada");
    s.confirm_delete();
    assert_eq!(s.shown(), &Shown::Nothing);
    assert!(s.notice().unwrap().contains("no longer exists"));
}
