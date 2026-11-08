//! A field the source judges: named as typed, and an invalid one never reaches the source.
//!
//! Tests: state machine tests, no window.

#[path = "tasks_common/mod.rs"]
mod tasks_common;

use std::sync::Arc;

use fs_components_workbenches::tasks::{TaskLabels, TasksState, Variant};
use tasks_common::Shelf;

#[test]
fn an_invalid_title_is_named_as_typed_and_blocks_the_save() {
    let shelf = Arc::new(Shelf::with(&["a"]));
    let mut s = TasksState::new(shelf.clone(), TaskLabels::default(), Variant::Full);
    s.begin_edit();
    assert_eq!(s.problem("title"), None, "what is stored is not judged");
    assert!(s.typed("title", "new!"), "valid to invalid: draw again");
    assert_eq!(s.problem("title").as_deref(), Some("no exclamation marks"));
    assert!(!s.typed("title", "new!!"), "same problem: no redraw");
    s.save();
    assert_eq!(shelf.writes(), 0, "nothing written");
    assert!(
        s.editing()
            .unwrap()
            .error
            .as_deref()
            .unwrap()
            .contains("not valid")
    );
    assert!(s.typed("title", "new"), "invalid to valid: draw again");
    s.save();
    assert!(s.editing().is_none());
    assert_eq!(shelf.writes(), 1);
    assert_eq!(shelf.title_of("a").unwrap(), "new");
}
