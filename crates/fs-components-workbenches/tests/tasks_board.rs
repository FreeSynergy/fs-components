//! The status view driven through Blinc's router and the keys a window sends. No window is opened.
//!
//! Tests: input tests. A hang here is a deadlock: run with a timeout.

#[path = "tasks_common/mod.rs"]
mod tasks_common;

use std::sync::Arc;

use fs_components_workbenches::tasks::{
    BoardOutcome, MoveOutcome, Place, TaskLabels, TaskView, TasksWidget,
};
use tasks_common::Shelf;
use tasks_common::harness::Harness;

const SPACE: u32 = 32;
const ESCAPE: u32 = 27;
const RIGHT: u32 = 39;
const DOWN: u32 = 40;

fn widget(shelf: Shelf) -> (Arc<Shelf>, TasksWidget) {
    let shelf = Arc::new(shelf);
    (
        shelf.clone(),
        TasksWidget::new(shelf, TaskLabels::default()),
    )
}

fn in_status(shelf: Shelf, width: f32) -> (Arc<Shelf>, Harness) {
    let (shelf, w) = widget(shelf);
    w.with_state(|s| s.set_view(TaskView::Status));
    (shelf, Harness::new(w, width, 700.0))
}

#[test]
fn a_click_on_a_card_opens_it() {
    let (_, mut h) = in_status(Shelf::with(&["a", "b"]), 1200.0);
    h.click("board-todo-b");
    assert_eq!(h.widget.context().selected.as_deref(), Some("b"));
}

#[test]
fn the_status_view_always_says_what_moving_does() {
    let (shelf, mut h) = in_status(Shelf::with(&["a"]), 1200.0);
    assert!(h.has("move-warning"), "before a move");
    h.click("move-done");
    assert_eq!(shelf.column_of("a").unwrap(), "done");
    assert!(
        h.has("move-warning"),
        "and after it: the next card may be a shared one"
    );
    assert!(h.has("notice"));
}

#[test]
fn a_double_click_on_a_move_button_goes_no_further_than_the_person_chose() {
    for rebuild in [false, true] {
        let (shelf, mut h) = in_status(Shelf::with(&["a"]), 1200.0);
        let point = h.centre("move-doing");
        h.press_at(point);
        if rebuild {
            h.rebuild();
        }
        h.press_at(point);
        h.rebuild();
        assert_eq!(shelf.column_of("a").unwrap(), "doing");
        assert_eq!(shelf.writes(), 1, "rebuild between: {rebuild}");
    }
}

#[test]
fn the_keyboard_moves_a_card_to_another_column_and_the_source_hears_it() {
    let (shelf, mut h) = in_status(Shelf::with(&["a", "b"]), 1200.0);
    assert!(h.tab_to("board"));
    h.key(SPACE);
    h.key(RIGHT);
    h.key(SPACE);
    assert_eq!(shelf.column_of("a").unwrap(), "doing");
    let moved = h.widget.with_state(|s| s.last_move().cloned());
    assert!(
        matches!(moved, Some(MoveOutcome::Moved { .. })),
        "{moved:?}"
    );
    h.rebuild();
    assert!(h.has("board-doing-a"));
}

#[test]
fn the_keyboard_can_move_a_second_card_after_the_window_rebuilt() {
    let (shelf, mut h) = in_status(Shelf::with(&["a", "b"]), 1200.0);
    assert!(h.tab_to("board"));
    h.key(SPACE);
    h.key(RIGHT);
    h.key(SPACE);
    h.rebuild();
    assert!(h.tab_to("board"), "the columns are a tab stop again");
    h.key(SPACE);
    h.key(RIGHT);
    h.key(SPACE);
    assert_eq!(shelf.column_of("a").unwrap(), "done");
}

#[test]
fn walking_over_cards_by_keyboard_opens_nothing_and_open_opens_the_cursor() {
    let shelf = Shelf::with(&["a", "b"]);
    shelf.guarded.lock().unwrap().push("a".into());
    let (_, mut h) = in_status(shelf, 1200.0);
    assert_eq!(h.widget.context().selected, None);
    assert!(h.tab_to("board"));
    h.key(DOWN);
    assert_eq!(h.widget.context().selected, None, "ADR 0029");
    assert_eq!(h.widget.board_cursor().as_deref(), Some("b"));
    h.press("open");
    assert_eq!(h.widget.context().selected.as_deref(), Some("b"));
}

#[test]
fn open_is_reachable_by_tab_and_enter() {
    let (_, mut h) = in_status(Shelf::with(&["a", "b"]), 1200.0);
    assert!(h.tab_to("board"));
    h.key(DOWN);
    assert!(h.tab_to("open"));
    h.key(13);
    assert_eq!(h.widget.context().selected.as_deref(), Some("b"));
}

#[test]
fn a_failed_move_by_keyboard_puts_the_card_back_and_says_why() {
    let (shelf, mut h) = in_status(Shelf::with(&["a"]), 1200.0);
    *shelf.refuse.lock().unwrap() = Some("disk full".into());
    assert!(h.tab_to("board"));
    h.key(SPACE);
    h.key(RIGHT);
    h.key(SPACE);
    h.rebuild();
    assert!(h.has("board-todo-a"), "back where it was");
    assert!(!h.has("board-doing-a"));
    assert!(h.has("notice"));
    let says = h.widget.with_state(|s| s.notice().map(str::to_string));
    assert!(says.unwrap().contains("disk full"));
}

#[test]
fn escape_puts_a_held_card_back_and_writes_nothing() {
    let (shelf, mut h) = in_status(Shelf::with(&["a"]), 1200.0);
    assert!(h.tab_to("board"));
    h.key(SPACE);
    h.key(RIGHT);
    h.key(ESCAPE);
    h.rebuild();
    assert_eq!(shelf.writes(), 0);
    assert!(h.has("board-todo-a"));
}

#[test]
fn a_card_put_elsewhere_in_its_own_column_changes_nothing_and_goes_back() {
    let (shelf, mut h) = in_status(Shelf::with(&["a", "b"]), 1200.0);
    assert!(h.tab_to("board"));
    h.key(SPACE);
    h.key(DOWN);
    h.key(SPACE);
    assert_eq!(shelf.writes(), 0);
    assert_eq!(h.widget.lanes()[0].items[0].key, "a", "as it was");
    assert!(h.widget.with_state(|s| s.notice().is_some()));
}

#[test]
fn a_card_that_cannot_move_cannot_be_picked_up_and_says_why() {
    let shelf = Shelf::with(&["a"]);
    shelf.fixed.lock().unwrap().push("a".into());
    let (shelf, mut h) = in_status(shelf, 1200.0);
    assert!(h.tab_to("board"));
    h.key(SPACE);
    h.key(RIGHT);
    h.key(SPACE);
    assert_eq!(shelf.writes(), 0);
    let says = h.widget.with_state(|s| s.notice().map(str::to_string));
    assert!(says.unwrap().contains("it repeats"));
    h.rebuild();
    assert!(!h.has("move-doing"), "and no button offers it");
}

#[test]
fn a_frontends_gesture_lands_in_the_same_place_as_the_keyboard() {
    let (shelf, h) = in_status(Shelf::with(&["a"]), 1200.0);
    let w = &h.widget;
    assert!(matches!(w.grab("a"), BoardOutcome::Grabbed(_)));
    assert!(matches!(
        w.hover(&Place::new("done", 0)),
        BoardOutcome::Carried
    ));
    assert_eq!(shelf.writes(), 0, "not until it is put down");
    assert!(matches!(w.drop_held(), BoardOutcome::Moved(_)));
    assert_eq!(shelf.column_of("a").unwrap(), "done");
}

#[test]
fn a_refused_grab_talks_about_the_card_grabbed_not_the_one_under_the_cursor() {
    let shelf = Shelf::with(&["a", "b"]);
    shelf.fixed.lock().unwrap().push("b".into());
    let (_, h) = in_status(shelf, 1200.0);
    assert_eq!(h.widget.board_cursor().as_deref(), Some("a"));
    assert!(matches!(h.widget.grab("b"), BoardOutcome::Refused(_)));
    let says = h.widget.with_state(|s| s.notice().map(str::to_string));
    assert!(says.unwrap().contains("it repeats"), "b's reason");
}

#[test]
fn leaving_the_view_gives_a_held_card_up_and_the_lanes_follow_again() {
    let (shelf, h) = in_status(Shelf::with(&["a", "b"]), 1200.0);
    let w = &h.widget;
    w.grab("a");
    w.hover(&Place::new("done", 0));
    w.with_state(|s| s.set_view(TaskView::List));
    assert_eq!(w.lanes()[0].items[0].key, "a", "back where it was");
    assert_eq!(shelf.writes(), 0);
    assert!(
        matches!(w.drop_held(), BoardOutcome::Refused(_)),
        "nothing held"
    );
    w.with_state(|s| s.search("b"));
    assert_eq!(w.lanes()[0].items.len(), 1, "the lanes follow");
}

#[test]
fn a_click_on_a_card_opens_it_once_and_moves_the_cursor_with_a_narrow_window_too() {
    for width in [1200.0, 480.0] {
        let (_, mut h) = in_status(Shelf::with(&["a", "b"]), width);
        h.click("board-todo-b");
        assert_eq!(h.widget.context().selected.as_deref(), Some("b"), "{width}");
        assert_eq!(h.widget.board_cursor().as_deref(), Some("b"), "{width}");
    }
}

#[test]
fn a_held_card_is_not_snatched_back_by_a_change_elsewhere() {
    let (shelf, h) = in_status(Shelf::with(&["a", "b"]), 1200.0);
    let w = &h.widget;
    w.grab("a");
    w.hover(&Place::new("done", 0));
    w.with_state(|s| {
        s.select("b");
    });
    assert_eq!(w.lanes()[2].items[0].key, "a", "still where it is held");
    assert!(matches!(w.drop_held(), BoardOutcome::Moved(_)));
    assert_eq!(shelf.column_of("a").unwrap(), "done");
    assert_eq!(w.lanes()[2].items[0].key, "a");
    assert_eq!(w.lanes()[0].items[0].key, "b");
}

#[test]
fn typing_into_the_search_field_narrows_the_columns_and_keeps_the_keyboard() {
    let (_, mut h) = in_status(Shelf::with(&["apple", "pear"]), 1200.0);
    h.press("search");
    assert!(h.type_text("pe"), "the field has the keyboard");
    h.frame();
    assert!(h.has("board-todo-pear"));
    assert!(!h.has("board-todo-apple"));
    assert!(h.type_text("a"), "and still has it");
    assert_eq!(h.widget.with_state(|s| s.term().to_string()), "pea");
}
