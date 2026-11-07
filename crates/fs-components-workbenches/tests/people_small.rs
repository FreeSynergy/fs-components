//! The people workbench at the smallest window, with the room a surface really has there, and a
//! double-click on Cancel. No window is opened; the wheel and the mouse go through the router.
//!
//! Tests: input tests. A hang here is a deadlock.

#[path = "people_common/mod.rs"]
mod people_common;

use std::sync::Arc;

use fs_components_app::min_surface;
use fs_components_workbenches::people::{PeopleLabels, PeopleWidget};
use people_common::Book;
use people_common::harness::Harness;

fn harness(book: Book, size: (f32, f32)) -> (Arc<Book>, Harness) {
    let book = Arc::new(book);
    let w = PeopleWidget::new(book.clone(), PeopleLabels::default());
    (book, Harness::new(w, size.0, size.1))
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

fn typed_name(h: &Harness) -> Option<String> {
    h.widget.with_state(|s| {
        s.editing().and_then(|e| {
            e.draft
                .fields()
                .find(|f| f.key == "name")
                .map(|f| f.value.clone())
        })
    })
}

#[test]
fn really_delete_and_discard_are_inside_the_surface_and_work() {
    let (book, mut h) = harness(Book::with(&["ada", "bob"]), min_surface());
    h.click("edit");
    h.widget.with_state(|s| s.set_field("name", "typed"));
    h.rebuild();
    h.click("cancel");
    inside(&h, "discard");
    h.click("discard");
    assert_eq!(book.name("ada").as_deref(), Some("ada"));
    h.click("delete");
    inside(&h, "really");
    h.click("really");
    assert!(book.name("ada").is_none());
}

#[test]
fn show_more_is_inside_the_surface() {
    let (_, h) = harness(Book::numbered(450), min_surface());
    inside(&h, "more");
}

#[test]
fn a_form_field_is_reached_by_the_wheel_and_takes_text() {
    let (_, mut h) = harness(Book::with(&["ada"]), min_surface());
    h.click("edit");
    let field = h.widget.with_state(|s| {
        let edit = s.editing().unwrap();
        edit.draft.fields().last().unwrap().key.clone()
    });
    let what = format!("field-{field}");
    let (_, height) = min_surface();
    for _ in 0..12 {
        if h.edges(&what).3 <= height {
            break;
        }
        h.wheel_at((4.0, height - 20.0), 60.0);
    }
    inside(&h, &what);
    h.press(&what);
    assert!(h.type_text("x"), "{field}");
    let got = h.widget.with_state(|s| {
        let edit = s.editing().unwrap();
        let found = edit.draft.fields().find(|f| f.key == field).unwrap();
        found.value.clone()
    });
    assert!(got.contains('x'));
}

#[test]
fn a_double_click_on_cancel_keeps_what_was_typed() {
    for rebuild in [false, true] {
        let (book, mut h) = harness(Book::with(&["ada"]), (1200.0, 700.0));
        h.click("edit");
        h.widget.with_state(|s| s.set_field("name", "typed"));
        h.rebuild();
        let point = h.centre("cancel");
        h.press_at(point);
        if rebuild {
            h.rebuild();
        }
        h.press_at(point);
        h.rebuild();
        assert_eq!(typed_name(&h).as_deref(), Some("typed"), "{rebuild}");
        if !h.has("discard") {
            h.click("cancel");
        }
        let (left, top, right, bottom) = h.edges("discard");
        let under = point.0 >= left && point.0 <= right && point.1 >= top && point.1 <= bottom;
        assert!(!under, "the first press does not land on it");
        h.click("discard");
        assert_eq!(book.name("ada").as_deref(), Some("ada"));
        assert!(typed_name(&h).is_none());
    }
}
