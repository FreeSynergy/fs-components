//! The workbench at every width, a failed Show more, and Show more from the keyboard.
//!
//! Tests: input tests over the router and the navigation keys a window uses. No window is opened.

#[path = "people_common/mod.rs"]
mod people_common;

use std::sync::Arc;

use fs_components_workbenches::people::{PeopleLabels, PeopleWidget};
use people_common::Book;
use people_common::harness::Harness;

fn labels() -> PeopleLabels {
    let mut labels = PeopleLabels::default();
    labels.columns = vec!["Name".into(), "E-mail".into()];
    labels
}

fn widget(book: Book) -> (Arc<Book>, PeopleWidget) {
    let book = Arc::new(book);
    (book.clone(), PeopleWidget::new(book, labels()))
}

fn rows_of(h: &Harness) -> usize {
    h.widget.with_state(|s| s.rows().len())
}

#[test]
fn really_delete_is_reachable_at_every_width_and_never_under_the_first_press() {
    for width in [1200.0, 900.0, 560.0, 500.0, 400.0, 300.0, 200.0, 160.0] {
        let (book, w) = widget(Book::with(&["ada", "bob"]));
        let mut h = Harness::new(w, width, 600.0);
        let point = h.centre("delete");
        h.click("delete");
        let (left, top, right, bottom) = h.edges("really");
        assert!(
            right <= width + 0.5 && left >= -0.5,
            "{width}: {left}..{right}"
        );
        assert!(bottom <= 600.0, "{width}: {top}..{bottom}");
        let under = point.0 >= left && point.0 <= right && point.1 >= top && point.1 <= bottom;
        assert!(!under, "{width}: the first press lands on it");
        h.click("really");
        assert!(book.name("ada").is_none(), "{width}: the click reached it");
    }
}

#[test]
fn a_double_press_keeps_the_person_at_a_narrow_width_too() {
    for width in [500.0, 300.0, 200.0] {
        let (book, w) = widget(Book::with(&["ada", "bob"]));
        let mut h = Harness::new(w, width, 600.0);
        let point = h.centre("delete");
        h.press_at(point);
        h.rebuild();
        h.press_at(point);
        h.rebuild();
        assert!(book.name("ada").is_some(), "{width}");
    }
}

#[test]
fn a_failed_show_more_keeps_the_rows_says_why_and_can_be_tried_again() {
    let (book, w) = widget(Book::numbered(450));
    let mut h = Harness::new(w, 900.0, 600.0);
    *book.broken_from.lock().unwrap() = Some(200);
    h.click("more");
    assert_eq!(rows_of(&h), 200, "the rows stay");
    assert!(h.widget.with_state(|s| s.list_problem().is_some()));
    assert!(h.has("p00000"), "and are drawn");
    assert!(h.has("more"), "and it can be tried again");
    *book.broken_from.lock().unwrap() = None;
    h.click("more");
    assert_eq!(rows_of(&h), 400);
    assert_eq!(
        h.widget
            .with_state(|s| s.list_problem().map(str::to_string)),
        None
    );
}

#[test]
fn a_failed_first_list_still_says_why() {
    let book = Book::with(&["ada"]);
    *book.broken_list.lock().unwrap() = Some("the disk is gone".into());
    let (_, w) = widget(book);
    let h = Harness::new(w, 900.0, 600.0);
    assert!(!h.has("more"));
    assert!(h.widget.with_state(|s| s.list_problem().is_some()));
}

#[test]
fn show_more_is_reached_by_tab_and_works_with_enter() {
    let (_, w) = widget(Book::numbered(450));
    let mut h = Harness::new(w, 900.0, 600.0);
    assert!(h.tab_to("more"), "Tab reaches it");
    h.press_key(13);
    assert_eq!(rows_of(&h), 400, "Enter works it");
}

#[test]
fn a_picture_at_a_narrow_width_with_the_question_open() {
    let _one = people_common::harness::serial();
    let (_, w) = widget(Book::with(&["ada", "bob"]));
    w.with_state(fs_components_workbenches::people::PeopleState::ask_delete);
    let path = std::env::var_os("FS_COMP_SHOTS").map_or_else(
        || std::env::temp_dir().join(format!("fs-narrow-{}.png", std::process::id())),
        |dir| std::path::PathBuf::from(dir).join("people-narrow.png"),
    );
    let canvas = fs_components_render::Canvas::new(300, 500, w.styles());
    let drawn = fs_components_render::draw_to_png(&canvas, |x, y| w.build(x, y), || {}, &path);
    if let Err(fs_components_render::RenderError::NoGraphics(why)) = &drawn {
        eprintln!("skipped: {why}");
        return;
    }
    drawn.unwrap();
    if std::env::var_os("FS_COMP_SHOTS").is_none() {
        std::fs::remove_file(&path).unwrap();
    }
}
