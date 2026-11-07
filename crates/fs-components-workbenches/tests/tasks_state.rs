//! The state machine of the tasks workbench, without a window.
//!
//! Tests: State tests over a source in memory.

#[path = "tasks_common/mod.rs"]
mod tasks_common;

use std::sync::Arc;

use fs_components_workbenches::tasks::{
    MoveOutcome, Outcome, Shown, TaskLabels, TaskView, TasksState, Variant,
};
use tasks_common::Shelf;

fn state(shelf: Shelf) -> (Arc<Shelf>, TasksState) {
    let shelf = Arc::new(shelf);
    let s = TasksState::new(shelf.clone(), TaskLabels::default(), Variant::Full);
    (shelf, s)
}

#[test]
fn it_opens_the_first_task_unless_that_one_is_only_to_be_opened_when_chosen() {
    let (_, s) = state(Shelf::with(&["a", "b"]));
    assert_eq!(s.selected(), Some("a"));
    let shelf = Shelf::with(&["a", "b"]);
    shelf.guarded.lock().unwrap().push("a".into());
    let (_, s) = state(shelf);
    assert_eq!(s.selected(), None, "ADR 0029");
    assert_eq!(s.rows()[0].marks, ["Sensitive"], "marked in the list");
}

#[test]
fn deleting_the_task_before_a_marked_one_does_not_open_it() {
    let shelf = Shelf::with(&["a", "b"]);
    shelf.guarded.lock().unwrap().push("b".into());
    let (_, mut s) = state(shelf);
    s.ask_delete();
    s.confirm_delete();
    assert_eq!(s.selected(), None);
}

#[test]
fn the_context_is_the_selected_task_and_the_view() {
    let (_, mut s) = state(Shelf::with(&["a", "b"]));
    let c = s.context();
    assert_eq!((c.selected.as_deref(), c.view), (Some("a"), TaskView::List));
    s.select("b");
    s.set_view(TaskView::Status);
    let c = s.context();
    assert_eq!(
        (c.selected.as_deref(), c.view),
        (Some("b"), TaskView::Status)
    );
}

#[test]
fn the_compact_variant_shows_open_tasks_and_changes_nothing() {
    let shelf = Arc::new(Shelf::with(&["a", "b"]));
    shelf.tasks.lock().unwrap().get_mut("b").unwrap().column = "done".into();
    let mut s = TasksState::new(shelf.clone(), TaskLabels::default(), Variant::Compact);
    assert_eq!(s.rows().len(), 1, "what is done is not next");
    s.set_view(TaskView::Status);
    assert_eq!(s.view(), TaskView::List);
    assert!(!s.can_edit() && !s.can_create() && !s.can_delete() && !s.can_move());
    assert!(matches!(s.move_task("a", "done"), MoveOutcome::Refused(_)));
    assert_eq!(shelf.writes(), 0);
}

#[test]
fn a_failed_show_more_keeps_the_rows_and_can_be_tried_again() {
    let (shelf, mut s) = state(Shelf::numbered(450));
    *shelf.broken_from.lock().unwrap() = Some(200);
    s.load_more();
    assert_eq!(s.rows().len(), 200);
    assert!(s.list_problem().is_some() && s.has_more());
    *shelf.broken_from.lock().unwrap() = None;
    s.load_more();
    assert_eq!(s.rows().len(), 400);
    assert!(s.list_problem().is_none());
}

#[test]
fn delete_takes_two_different_actions_tied_to_one_task() {
    let (shelf, mut s) = state(Shelf::with(&["a", "b"]));
    s.ask_delete();
    s.ask_delete();
    assert!(
        shelf.title_of("a").is_some(),
        "asking twice does not delete"
    );
    s.select("b");
    assert!(!s.delete_asked(), "the question stays with its task");
    s.confirm_delete();
    assert_eq!(shelf.writes(), 0, "not asked about this one");
    s.ask_delete();
    s.confirm_delete();
    assert!(shelf.title_of("b").is_none());
}

#[test]
fn a_save_of_a_task_someone_else_deleted_keeps_what_was_typed() {
    let (shelf, mut s) = state(Shelf::with(&["a", "b"]));
    s.begin_edit();
    s.set_field("title", "typed");
    shelf.other_deletes("a");
    s.save();
    let edit = s.editing().expect("the draft stays");
    assert_eq!(edit.draft.value("title"), Some("typed"));
    assert_eq!(edit.error.as_deref(), Some(s.labels().gone_draft.as_str()));
    assert_eq!(s.rows().len(), 1, "the task left the list");
    s.cancel_edit();
    s.confirm_discard();
    assert_eq!(s.selected(), None);
    assert_eq!(s.notice(), Some(s.labels().gone.as_str()));
}

#[test]
fn walking_over_rows_opens_nothing_and_reads_nothing() {
    let shelf = Shelf::with(&["a", "b"]);
    shelf.guarded.lock().unwrap().push("b".into());
    let (_, mut s) = state(shelf);
    s.walk("b");
    assert_eq!(s.list_cursor(), Some("b"));
    assert_eq!(s.selected(), Some("a"), "still the one chosen");
    assert_eq!(s.select("b"), Outcome::Done);
    assert_eq!(s.selected(), Some("b"));
    assert_eq!(s.list_cursor(), Some("b"));
}

#[test]
fn a_task_someone_else_deleted_leaves_the_view() {
    let (shelf, mut s) = state(Shelf::with(&["a", "b"]));
    s.begin_edit();
    shelf.other_deletes("a");
    s.cancel_edit();
    assert_eq!(s.selected(), None, "nothing typed: Cancel leaves at once");
    assert_eq!(s.rows().len(), 1);
    assert_eq!(s.notice(), Some(s.labels().gone.as_str()));
}

#[test]
fn unsaved_changes_block_leaving_and_the_first_cancel_only_asks() {
    let (shelf, mut s) = state(Shelf::with(&["a", "b"]));
    s.begin_edit();
    s.set_field("title", "new");
    assert_eq!(s.select("b"), Outcome::Blocked);
    assert_eq!(s.begin_create(), Outcome::Blocked);
    assert_eq!(s.move_task("b", "done"), MoveOutcome::Blocked);
    assert_eq!(shelf.writes(), 0);
    s.cancel_edit();
    s.cancel_edit();
    assert!(s.editing().is_some(), "a double-click only asks");
    assert_eq!(s.editing().unwrap().draft.value("title"), Some("new"));
    s.keep_editing();
    s.confirm_discard();
    assert!(s.editing().is_some(), "not asked any more");
    s.cancel_edit();
    s.set_field("title", "newer");
    s.confirm_discard();
    assert!(
        s.editing().is_some(),
        "typing since then takes the question back"
    );
    s.cancel_edit();
    s.confirm_discard();
    assert!(s.editing().is_none());
    assert_eq!(shelf.title_of("a").unwrap(), "A");
}

#[test]
fn a_failed_save_keeps_the_draft_and_reads_the_list_again() {
    let (shelf, mut s) = state(Shelf::with(&["a", "b"]));
    s.begin_edit();
    s.set_field("title", "new");
    *shelf.refuse.lock().unwrap() = Some("disk full".into());
    shelf.other_changes("b", "B2");
    s.save();
    let edit = s.editing().expect("still editing");
    assert_eq!(edit.draft.value("title"), Some("new"));
    assert!(edit.error.as_deref().unwrap().contains("disk full"));
    assert!(s.rows().iter().any(|r| r.title == "B2"), "read again");
}

#[test]
fn after_a_conflict_the_next_save_writes_knowingly() {
    let (shelf, mut s) = state(Shelf::with(&["a"]));
    s.begin_edit();
    s.set_field("title", "mine");
    shelf.other_changes("a", "theirs");
    s.save();
    assert_eq!(shelf.title_of("a").unwrap(), "theirs", "nothing written");
    assert!(s.editing().unwrap().error.is_some());
    s.save();
    assert_eq!(shelf.title_of("a").unwrap(), "mine");
}

#[test]
fn a_save_without_changes_writes_nothing_and_reads_again() {
    let (shelf, mut s) = state(Shelf::with(&["a"]));
    s.begin_edit();
    shelf.other_changes("a", "theirs");
    s.save();
    assert_eq!(shelf.writes(), 0);
    assert!(s.editing().is_none());
    assert!(matches!(s.shown(), Shown::Task { sheet, .. } if sheet.title == "theirs"));
    assert!(s.rows().iter().any(|r| r.title == "theirs"));
}

#[test]
fn a_list_that_cannot_be_read_again_stays_as_drawn_and_says_why_but_a_new_search_starts_empty() {
    let (shelf, mut s) = state(Shelf::with(&["a", "b"]));
    *shelf.broken_from.lock().unwrap() = Some(0);
    s.begin_edit();
    s.save();
    assert_eq!(
        s.rows().len(),
        2,
        "a failed read says nothing about the tasks"
    );
    assert!(s.list_problem().is_some());
    s.search("a");
    assert!(s.rows().is_empty(), "these are not the rows of this search");
    assert!(s.list_problem().is_some());
}
