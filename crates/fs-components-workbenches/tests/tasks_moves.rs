//! Moves in the state machine of the tasks workbench, without a window.
//!
//! Pattern: State tests over a source in memory.

#[path = "tasks_common/mod.rs"]
mod tasks_common;

use std::sync::Arc;

use fs_components_workbenches::tasks::{MoveOutcome, TaskLabels, TasksState, Variant};
use tasks_common::Shelf;

fn state(shelf: Shelf) -> (Arc<Shelf>, TasksState) {
    let shelf = Arc::new(shelf);
    let s = TasksState::new(shelf.clone(), TaskLabels::default(), Variant::Full);
    (shelf, s)
}

#[test]
fn a_move_answers_with_a_value_and_says_so_for_everyone() {
    let (shelf, mut s) = state(Shelf::with(&["a"]));
    assert!(!s.moved_once());
    let done = s.move_task("a", "done");
    assert_eq!(
        done,
        MoveOutcome::Moved {
            key: "a".into(),
            from: "todo".into(),
            to: "done".into()
        }
    );
    assert_eq!(s.last_move(), Some(&done));
    assert!(s.moved_once());
    assert_eq!(shelf.column_of("a").unwrap(), "done");
    assert_eq!(s.notice(), Some("A now has the status Done, for everyone."));
}

#[test]
fn a_move_to_the_column_it_is_in_writes_nothing() {
    let (shelf, mut s) = state(Shelf::with(&["a"]));
    assert_eq!(s.move_task("a", "todo"), MoveOutcome::Unchanged);
    assert_eq!(shelf.writes(), 0);
    assert!(!s.moved_once());
}

#[test]
fn a_failed_move_leaves_the_task_says_why_and_shows_what_is_there() {
    let (shelf, mut s) = state(Shelf::with(&["a"]));
    *shelf.refuse.lock().unwrap() = Some("disk full".into());
    assert!(matches!(s.move_task("a", "done"), MoveOutcome::Refused(_)));
    assert_eq!(s.rows()[0].column, "todo");
    assert!(s.notice().unwrap().contains("disk full"));
    assert!(!s.moved_once());
}

#[test]
fn a_move_after_someone_else_changed_the_task_is_a_conflict_and_shows_their_change() {
    let (shelf, mut s) = state(Shelf::with(&["a"]));
    shelf.other_changes("a", "theirs");
    assert!(matches!(s.move_task("a", "done"), MoveOutcome::Conflict(_)));
    assert_eq!(shelf.column_of("a").unwrap(), "todo");
    assert_eq!(s.rows()[0].title, "theirs");
}

#[test]
fn a_move_of_a_task_that_is_gone_takes_it_out_of_the_view() {
    let (shelf, mut s) = state(Shelf::with(&["a", "b"]));
    shelf.other_deletes("a");
    assert_eq!(s.move_task("a", "done"), MoveOutcome::Gone);
    assert_eq!(s.rows().len(), 1);
    assert_eq!(s.selected(), None);
}
