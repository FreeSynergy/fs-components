//! The tasks workbench at the smallest window, with the room a surface really has there. No window is
//! opened; the wheel, the mouse and the keys go through the router.
//!
//! Pattern: input tests. A hang here is a deadlock.

#[path = "tasks_common/mod.rs"]
mod tasks_common;

use std::sync::Arc;

use fs_components_app::min_surface;
use fs_components_workbenches::tasks::{TaskLabels, TaskView, TasksWidget};
use tasks_common::Shelf;
use tasks_common::harness::Harness;

fn harness(shelf: Shelf, view: TaskView) -> (Arc<Shelf>, Harness) {
    let shelf = Arc::new(shelf);
    let w = TasksWidget::new(shelf.clone(), TaskLabels::default());
    w.with_state(|s| s.set_view(view));
    let (width, height) = min_surface();
    (shelf, Harness::new(w, width, height))
}

fn inside(h: &Harness, what: &str) {
    let (width, height) = min_surface();
    let (left, top, right, bottom) = h.edges(what);
    assert!(
        left >= -0.5 && right <= width + 0.5,
        "{what}: {left}..{right}"
    );
    assert!(
        top >= -0.5 && bottom <= height + 0.5,
        "{what}: {top}..{bottom}"
    );
}

#[test]
fn really_delete_and_discard_are_inside_the_surface_and_work() {
    for view in [TaskView::List, TaskView::Status] {
        let (shelf, mut h) = harness(Shelf::with(&["a", "b"]), view);
        h.click("edit");
        h.widget.with_state(|s| s.set_field("title", "typed"));
        h.rebuild();
        h.click("cancel");
        inside(&h, "discard");
        h.click("discard");
        assert_eq!(shelf.title_of("a").unwrap(), "A");
        h.click("delete");
        inside(&h, "really");
        h.click("really");
        assert!(shelf.title_of("a").is_none(), "{view:?}");
    }
}

#[test]
fn show_more_is_inside_the_surface_in_both_views() {
    for view in [TaskView::List, TaskView::Status] {
        let (_, h) = harness(Shelf::numbered(450), view);
        inside(&h, "more");
    }
}

#[test]
fn every_form_field_is_reached_by_the_wheel_and_takes_text() {
    for field in ["title", "notes"] {
        let (_, mut h) = harness(Shelf::with(&["a"]), TaskView::List);
        h.click("edit");
        let (_, height) = min_surface();
        // In the page's margin: a text area under the pointer takes the wheel for itself, also when it
        // has nothing to scroll (see the Blinc notes in `DESIGN.md`).
        let over = (4.0, height - 20.0);
        for _ in 0..12 {
            let (_, top, _, bottom) = h.edges(&format!("field-{field}"));
            if top >= height / 2.0 && bottom <= height {
                break;
            }
            h.wheel_at(over, 60.0);
        }
        inside(&h, &format!("field-{field}"));
        h.press(&format!("field-{field}"));
        assert!(h.type_text("x"), "{field}");
        let got = h.widget.with_state(|s| {
            s.editing()
                .and_then(|e| e.draft.value(field).map(str::to_string))
        });
        assert!(got.unwrap().contains('x'), "{field}");
    }
}

#[test]
fn the_page_starts_at_the_top_for_another_task() {
    let (_, mut h) = harness(Shelf::with(&["a", "b"]), TaskView::List);
    h.click("edit");
    let (width, height) = min_surface();
    h.wheel_at((width / 2.0, height - 20.0), 100.0);
    h.click("cancel");
    h.click("b");
    inside(&h, "edit");
}
