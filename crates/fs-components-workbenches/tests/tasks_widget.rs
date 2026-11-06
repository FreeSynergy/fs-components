//! The workbench driven through Blinc's router and the keys a window sends. No window is opened.
//!
//! Pattern: input tests. A hang here is a deadlock: run with a timeout.

#[path = "tasks_common/mod.rs"]
mod tasks_common;

use std::sync::Arc;

use fs_components_workbenches::tasks::{TaskLabels, TaskView, TasksWidget};
use tasks_common::Shelf;
use tasks_common::harness::Harness;

fn widget(shelf: Shelf) -> (Arc<Shelf>, TasksWidget) {
    let shelf = Arc::new(shelf);
    (
        shelf.clone(),
        TasksWidget::new(shelf, TaskLabels::default()),
    )
}

#[test]
fn a_click_on_a_row_opens_it_and_the_context_says_so() {
    let (_, w) = widget(Shelf::with(&["a", "b"]));
    let mut h = Harness::new(w, 1200.0, 700.0);
    h.click("b");
    assert_eq!(h.widget.context().selected.as_deref(), Some("b"));
    h.click("view-status");
    assert_eq!(h.widget.context().view, TaskView::Status);
    h.click("view-list");
    assert_eq!(h.widget.context().view, TaskView::List);
}

#[test]
fn delete_is_reachable_at_every_width_and_a_double_press_never_deletes() {
    for view in [TaskView::List, TaskView::Status] {
        for width in [1200.0, 900.0, 720.0, 560.0, 400.0, 300.0, 200.0, 160.0] {
            for rebuild in [false, true] {
                let (shelf, w) = widget(Shelf::with(&["a", "b"]));
                w.with_state(|s| s.set_view(view));
                let mut h = Harness::new(w, width, 700.0);
                let point = h.centre("delete");
                h.press_at(point);
                if rebuild {
                    h.rebuild();
                }
                h.press_at(point);
                h.rebuild();
                assert!(shelf.title_of("a").is_some(), "{width} {rebuild}");
                assert_eq!(shelf.writes(), 0);
                // After a redraw the second press landed on Keep, which is the point of it.
                if !h.has("really") {
                    h.click("delete");
                }
                let (left, top, right, bottom) = h.edges("really");
                assert!(
                    right <= width + 0.5 && left >= -0.5,
                    "{width}: {left}..{right}"
                );
                assert!(bottom <= 700.0, "{width}: {top}..{bottom}");
                let under =
                    point.0 >= left && point.0 <= right && point.1 >= top && point.1 <= bottom;
                assert!(!under, "{width}: the first press lands on it");
                h.click("really");
                assert!(
                    shelf.title_of("a").is_none(),
                    "{width}: the click reached it"
                );
            }
        }
    }
}

#[test]
fn the_toolbar_is_inside_the_width_at_every_width() {
    for width in [1200.0, 560.0, 400.0, 300.0, 200.0, 160.0] {
        let (_, w) = widget(Shelf::with(&["a"]));
        let h = Harness::new(w, width, 700.0);
        for what in ["view-list", "view-status", "create"] {
            let (left, _, right, bottom) = h.edges(what);
            assert!(
                left >= -0.5 && right <= width + 0.5,
                "{width} {what}: {left}..{right}"
            );
            assert!(bottom <= 700.0, "{width} {what}");
        }
    }
}

#[test]
fn typing_through_real_key_events_reaches_the_draft() {
    let (shelf, w) = widget(Shelf::with(&["a"]));
    let mut h = Harness::new(w, 1200.0, 700.0);
    h.click("edit");
    h.press("field-title");
    assert!(h.type_text("xy"), "the field has the focus after a click");
    let value = h.widget.with_state(|s| {
        s.editing()
            .and_then(|e| e.draft.value("title").map(str::to_string))
    });
    // The click puts the cursor at the start here: the router gives no text-relative position.
    assert_eq!(value.as_deref(), Some("xyA"));
    h.click("save");
    assert_eq!(shelf.title_of("a").unwrap(), "xyA");
}

#[test]
fn a_choice_in_the_form_is_kept_with_what_was_typed() {
    let (shelf, w) = widget(Shelf::with(&["a"]));
    let mut h = Harness::new(w, 1200.0, 700.0);
    h.click("edit");
    h.press("field-title");
    h.type_text("z");
    h.click("field-column-doing");
    h.click("save");
    assert_eq!(shelf.title_of("a").unwrap(), "zA");
    assert_eq!(shelf.column_of("a").unwrap(), "doing");
}

#[test]
fn show_more_is_reached_by_tab_and_enter_in_both_views_and_a_failure_keeps_the_rows() {
    for view in [TaskView::List, TaskView::Status] {
        let (shelf, w) = widget(Shelf::numbered(450));
        w.with_state(|s| s.set_view(view));
        let mut h = Harness::new(w, 1200.0, 700.0);
        *shelf.broken_from.lock().unwrap() = Some(200);
        assert!(h.tab_to("more"), "{view:?}");
        h.key(13);
        h.rebuild();
        let rows = h.widget.with_state(|s| s.rows().len());
        assert_eq!(rows, 200, "kept");
        assert!(h.has("more"), "and it can be tried again");
        *shelf.broken_from.lock().unwrap() = None;
        assert!(h.tab_to("more"));
        h.key(13);
        let rows = h.widget.with_state(|s| s.rows().len());
        assert_eq!(rows, 400, "{view:?}");
    }
}

#[test]
fn the_compact_variant_offers_no_change_and_no_second_view() {
    let shelf = Arc::new(Shelf::with(&["a"]));
    let w = TasksWidget::compact(shelf, TaskLabels::default());
    let h = Harness::new(w, 400.0, 500.0);
    for what in ["view-status", "create", "edit", "delete", "move-done"] {
        assert!(!h.has(what), "{what}");
    }
    assert_eq!(h.widget.context().selected.as_deref(), Some("a"));
}

#[test]
fn typing_into_a_long_text_through_real_key_events_reaches_the_draft() {
    let (_, w) = widget(Shelf::with(&["a"]));
    let mut h = Harness::new(w, 1200.0, 700.0);
    h.click("edit");
    h.press("field-notes");
    assert!(h.type_text("hello"), "the area has the focus after a click");
    let value = h.widget.with_state(|s| {
        s.editing()
            .and_then(|e| e.draft.value("notes").map(str::to_string))
    });
    assert_eq!(value.as_deref(), Some("hello"));
}
