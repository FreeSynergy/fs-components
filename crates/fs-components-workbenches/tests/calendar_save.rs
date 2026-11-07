//! Saving in the calendar workbench: what is said when nothing was changed.
//!
//! Tests: State machine tests, no window.

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

#[test]
fn saving_without_a_change_after_a_delete_elsewhere_says_gone_not_nothing_changed() {
    let (diary, mut s) = state(Diary::with(&[("ada", "2026-11-04"), ("bob", "2026-11-20")]));
    s.select("ada");
    s.begin_edit();
    diary.other_deletes("ada");
    s.save();
    assert_eq!(s.shown(), &Shown::Nothing);
    assert!(
        s.notice().unwrap().contains("no longer exists"),
        "{:?}",
        s.notice()
    );
}

#[test]
fn saving_a_new_entry_back_to_its_blank_state_says_nothing_changed_not_the_old_refusal() {
    let (diary, mut s) = state(Diary::with(&[("ada", "2026-11-04")]));
    assert_eq!(s.begin_create(), Outcome::Done);
    let blank = s.editing().unwrap().draft.fields[0].value.clone();
    s.set_field("title", "Typed");
    assert_eq!(s.select("ada"), Outcome::Blocked);
    assert_eq!(s.notice(), Some("Save or cancel your changes first."));
    s.set_field("title", &blank);
    s.save();
    assert_eq!(diary.writes(), 0);
    assert_eq!(s.notice(), Some("Nothing was changed."));
}
