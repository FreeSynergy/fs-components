//! The state machine of the calendar workbench: going elsewhere, editing, deleting.
//!
//! Pattern: State machine tests, no window.

#[path = "calendar_common/mod.rs"]
mod calendar_common;

use std::sync::Arc;

use calendar_common::{Diary, d};
use fs_components_workbenches::calendar::{
    CalendarCapabilities, CalendarLabels, CalendarState, LAST_YEAR, Outcome, Picked, Shown,
    Variant, ViewRange,
};

fn state(diary: Diary) -> (Arc<Diary>, CalendarState) {
    let diary = Arc::new(diary);
    let state = CalendarState::new(diary.clone(), CalendarLabels::default(), Variant::Full);
    (diary, state)
}

fn pair() -> Diary {
    Diary::with(&[("ada", "2026-11-04"), ("bob", "2026-11-20")])
}

#[test]
fn it_opens_at_today_with_nothing_open_even_when_something_is_marked() {
    let diary = pair();
    diary.marked.lock().unwrap().push("ada".into());
    let (_, s) = state(diary);
    assert_eq!(s.anchor(), d("2026-11-06"));
    assert_eq!(s.view(), ViewRange::Month);
    assert_eq!(s.items().len(), 2);
    assert_eq!(s.items()[0].marks, ["Sensitive"], "marked where it appears");
    assert_eq!(s.shown(), &Shown::Nothing, "nothing opens on its own");
}

#[test]
fn navigation_moves_the_span_and_only_a_views_span_is_asked_for() {
    let (diary, mut s) = state(pair());
    s.next();
    assert_eq!(s.anchor(), d("2026-12-06"));
    s.previous();
    s.previous();
    assert_eq!(s.anchor(), d("2026-10-06"));
    s.go_today();
    assert_eq!(s.anchor(), d("2026-11-06"));
    assert_eq!(s.selected_day(), Some(d("2026-11-06")));
    s.set_view(ViewRange::Week);
    assert_eq!(s.span(), (d("2026-11-02"), d("2026-11-08")));
    s.next();
    assert_eq!(s.anchor(), d("2026-11-13"));
    s.set_view(ViewRange::Day);
    s.previous();
    assert_eq!(s.span(), (d("2026-11-12"), d("2026-11-12")));
    s.set_view(ViewRange::Agenda);
    assert_eq!(s.span(), (d("2026-11-12"), d("2026-12-11")));
    for (first, last) in diary.asked.lock().unwrap().iter() {
        assert!(last.number() - first.number() <= 41, "{first} {last}");
    }
}

#[test]
fn the_week_starts_where_the_source_says() {
    let mut diary = pair();
    diary.week_start = 6;
    let (_, mut s) = state(diary);
    s.set_view(ViewRange::Week);
    assert_eq!(s.span().0, d("2026-11-01"));
}

#[test]
fn a_far_future_date_works_and_the_calendar_says_where_it_stops() {
    let (_, mut s) = state(pair());
    s.go_to(d("2999-12-31"));
    s.next();
    assert_eq!(s.anchor(), d("3000-01-31"));
    s.go_to(d("9000-12-15"));
    s.next();
    assert_eq!(s.anchor(), d("9000-12-15"), "stays");
    assert!(s.notice().unwrap().contains(&LAST_YEAR.to_string()));
}

#[test]
fn a_click_on_a_day_chooses_it_and_a_count_opens_the_day_then_the_agenda() {
    let (_, mut s) = state(pair());
    s.pick(&Picked::Day(d("2026-11-11")));
    assert_eq!(s.selected_day(), Some(d("2026-11-11")));
    assert_eq!(s.view(), ViewRange::Month);
    s.pick(&Picked::More(d("2026-11-20")));
    assert_eq!((s.view(), s.anchor()), (ViewRange::Day, d("2026-11-20")));
    s.pick(&Picked::More(d("2026-11-20")));
    assert_eq!(s.view(), ViewRange::Agenda);
}

#[test]
fn a_new_item_starts_on_the_day_chosen_and_the_calendar_follows_it() {
    let (diary, mut s) = state(pair());
    s.pick(&Picked::Day(d("2026-11-11")));
    assert_eq!(s.begin_create(), Outcome::Done);
    assert_eq!(s.editing().unwrap().draft.value("day"), Some("2026-11-11"));
    s.set_field("title", "New one");
    s.set_field("day", "2027-02-03");
    s.save();
    assert_eq!(diary.title("New one").as_deref(), Some("New one"));
    assert_eq!(s.selected(), Some("New one"));
    assert_eq!(s.anchor(), d("2027-02-03"), "the calendar went there");
    assert!(s.items().iter().any(|i| i.key == "New one"));
    assert_eq!(s.notice(), Some("Created."));
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

#[test]
fn the_compact_variant_reads_only_and_starts_as_an_agenda() {
    let diary = Arc::new(pair());
    let mut s = CalendarState::new(diary, CalendarLabels::default(), Variant::Compact);
    assert_eq!(s.view(), ViewRange::Agenda);
    assert!(!s.can_create());
    s.select("bob");
    assert_eq!(s.selected(), Some("bob"));
    assert!(!s.can_edit() && !s.can_delete());
    s.ask_delete();
    s.begin_edit();
    assert!(s.editing().is_none() && !s.delete_asked());
}

#[test]
fn a_read_only_source_is_offered_nothing_that_changes() {
    let mut diary = pair();
    diary.capabilities = CalendarCapabilities::read_only();
    let (_, mut s) = state(diary);
    s.select("ada");
    assert!(!s.can_create() && !s.can_edit() && !s.can_delete());
    assert_eq!(s.begin_create(), Outcome::Done);
    assert!(s.editing().is_none());
}
