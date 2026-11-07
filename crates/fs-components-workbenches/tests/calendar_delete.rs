//! The state machine of the calendar workbench: deleting.
//!
//! Tests: State machine tests, no window.

#[path = "calendar_common/mod.rs"]
mod calendar_common;

use std::sync::Arc;

use calendar_common::Diary;
use fs_components_workbenches::calendar::{CalendarLabels, CalendarState, Shown, Variant};

fn state(diary: Diary) -> (Arc<Diary>, CalendarState) {
    let diary = Arc::new(diary);
    let state = CalendarState::new(diary.clone(), CalendarLabels::default(), Variant::Full);
    (diary, state)
}

fn pair() -> Diary {
    Diary::with(&[("ada", "2026-11-04"), ("bob", "2026-11-20")])
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

#[test]
fn what_was_marked_is_not_named_after_its_delete() {
    let diary = pair();
    diary.marked.lock().unwrap().push("ada".into());
    let (_, mut s) = state(diary);
    s.select("ada");
    s.ask_delete();
    s.confirm_delete();
    let notice = s.notice().unwrap();
    assert!(!notice.contains("ada"), "{notice}");
    assert_eq!(notice, "The item was deleted.");
}

#[test]
fn the_delete_question_about_something_unreadable_says_what_goes_with_it() {
    let (_, mut s) = state(pair());
    s.select("broken-series#2026-11-04");
    assert!(matches!(s.shown(), Shown::Failed { .. }));
    s.ask_delete();
    assert!(
        s.notice().unwrap().contains("All of the series goes."),
        "{:?}",
        s.notice()
    );
}
