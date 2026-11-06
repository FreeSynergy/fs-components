//! `PeopleWidget`: the real element, clicked and typed into in-process over the router and hit test
//! a window uses, and drawn offscreen.
//!
//! Pattern: input tests + pixel tests (skipped with a message without a graphics adapter). No window
//! is opened. A hang here is a deadlock, not slowness.

#[path = "people_common/mod.rs"]
mod people_common;

use std::path::PathBuf;
use std::sync::Arc;

use fs_components_render::{Canvas, RenderError, draw_to_png};
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

fn selected_of(widget: &PeopleWidget) -> Option<String> {
    widget.with_state(|s| s.selected().map(str::to_string))
}

fn selected(harness: &Harness) -> Option<String> {
    harness
        .widget
        .with_state(|s| s.selected().map(str::to_string))
}

#[test]
fn a_click_on_a_row_selects_and_builds_again_without_a_deadlock() {
    let (_, w) = widget(Book::with(&["ada", "bob"]));
    let mut h = Harness::new(w, 900.0, 600.0);
    assert_eq!(selected(&h).as_deref(), Some("ada"));
    h.click("bob");
    assert_eq!(selected(&h).as_deref(), Some("bob"));
}

#[test]
fn edit_type_save_by_mouse_and_keyboard() {
    let (book, w) = widget(Book::with(&["ada", "bob"]));
    let mut h = Harness::new(w, 900.0, 600.0);
    h.click("edit");
    assert!(h.widget.with_state(|s| s.editing().is_some()));
    assert!(h.has("save") && !h.has("edit") && !h.has("delete"));
    h.widget.with_state(|s| s.set_field("name", "Ada L."));
    h.click("save");
    assert_eq!(book.name("ada").as_deref(), Some("Ada L."));
    assert!(h.has("edit"));
}

#[test]
fn the_text_of_a_field_survives_a_redraw_during_the_edit() {
    let (_, w) = widget(Book::with(&["ada"]));
    let mut h = Harness::new(w, 900.0, 600.0);
    h.click("edit");
    h.widget.with_state(|s| s.set_field("name", "typed"));
    h.click("add-mail");
    assert_eq!(
        h.widget.with_state(|s| s
            .editing()
            .unwrap()
            .draft
            .fields()
            .next()
            .unwrap()
            .value
            .clone()),
        "typed"
    );
    assert_eq!(
        h.widget
            .with_state(|s| s.editing().unwrap().draft.fields().count()),
        3,
        "the field was added by the click"
    );
}

#[test]
fn delete_takes_two_clicks_and_the_button_says_so() {
    let (book, w) = widget(Book::with(&["ada", "bob"]));
    let mut h = Harness::new(w, 900.0, 600.0);
    h.click("delete");
    assert!(h.has("keep"));
    assert!(book.name("ada").is_some());
    h.click("really");
    assert!(book.name("ada").is_none());
    assert_eq!(selected(&h).as_deref(), Some("bob"));
    assert!(!h.has("keep"));
}

#[test]
fn a_double_click_on_delete_deletes_nothing_with_or_without_a_redraw_between() {
    let (book, w) = widget(Book::with(&["ada", "bob"]));
    let mut h = Harness::new(w, 900.0, 600.0);
    let point = h.centre("delete");
    // A window that has not drawn again: the same button is pressed twice.
    h.press_at(point);
    h.press_at(point);
    assert!(book.name("ada").is_some());
    h.rebuild();
    assert!(h.has("keep"), "still only asking");
    assert!(
        h.edges("really").1 > point.1,
        "the deed is below the pointer"
    );
    // A window that has: what is under the pointer now keeps the person.
    h.press_at(point);
    h.rebuild();
    assert!(book.name("ada").is_some());
    assert!(
        h.has("delete") && !h.has("keep"),
        "the second press said no"
    );
    assert_eq!(selected(&h).as_deref(), Some("ada"));
}

#[test]
fn someone_marked_is_marked_in_the_list_and_not_opened_by_the_workbench() {
    let book = Book::with(&["ada", "bob"]);
    book.guarded.lock().unwrap().push("ada".into());
    let (book, w) = widget(book);
    assert_eq!(selected_of(&w), None, "ada is first and is not opened");
    w.with_state(|s| s.select("bob"));
    assert_eq!(selected_of(&w).as_deref(), Some("bob"));
    assert!(w.with_state(|s| s.rows()[0].marks == ["Sensitive"]));
    w.with_state(|s| s.select("ada"));
    assert_eq!(selected_of(&w).as_deref(), Some("ada"), "chosen, so shown");
    book.guarded.lock().unwrap().push("cid".into());
    let compact = PeopleWidget::compact(book, labels());
    assert_eq!(selected_of(&compact), None, "the compact variant too");
}

#[test]
fn after_a_delete_the_next_person_is_not_opened_if_marked() {
    let book = Book::with(&["ada", "bob", "cid"]);
    book.guarded.lock().unwrap().push("bob".into());
    let (_, w) = widget(book);
    assert_eq!(selected_of(&w).as_deref(), Some("ada"));
    w.with_state(|s| {
        s.ask_delete();
        s.confirm_delete();
    });
    assert_eq!(selected_of(&w), None, "bob moved up and stays closed");
    assert!(w.with_state(|s| s.notice().unwrap().contains("ada")));
}

#[test]
fn show_more_and_the_height_of_the_list_follow_a_search() {
    let (_, w) = widget(Book::numbered(450));
    let mut h = Harness::new(w, 900.0, 600.0);
    assert!(h.has("more"));
    let before = h.height_of_list();
    h.press("search");
    assert!(h.type_text("p00001"));
    h.frame();
    assert!(!h.has("more"), "one person found, nothing more to show");
    assert!(
        h.height_of_list() > before,
        "the room of the button goes to the list"
    );
}

#[test]
fn typing_in_the_search_field_shortens_the_list_and_keeps_the_rest() {
    let (_, w) = widget(Book::with(&["ada", "bob", "bert"]));
    let mut h = Harness::new(w, 900.0, 600.0);
    h.press("search");
    assert!(h.type_text("b"), "the field has the keyboard after a click");
    assert_eq!(h.widget.with_state(|s| s.term().to_string()), "b");
    h.frame();
    assert!(!h.has("ada"), "ada left the list");
    assert!(h.has("bob") && h.has("bert"));
    assert_eq!(selected(&h).as_deref(), Some("ada"), "she is still shown");
}

#[test]
fn more_brings_more_rows() {
    let (_, w) = widget(Book::numbered(450));
    let mut h = Harness::new(w, 900.0, 600.0);
    assert_eq!(h.widget.with_state(|s| s.rows().len()), 200);
    h.click("more");
    assert_eq!(h.widget.with_state(|s| s.rows().len()), 400);
    h.click("more");
    assert_eq!(h.widget.with_state(|s| s.rows().len()), 450);
    assert!(!h.has("more"));
}

#[test]
fn the_compact_variant_offers_nothing_that_changes() {
    let book = Arc::new(Book::with(&["ada", "bob"]));
    let mut h = Harness::new(PeopleWidget::compact(book, labels()), 500.0, 400.0);
    assert!(h.has("ada") && !h.has("create") && !h.has("edit") && !h.has("delete"));
    h.click("bob");
    assert_eq!(selected(&h).as_deref(), Some("bob"));
    assert!(!h.has("edit") && !h.has("delete"));
}

#[test]
fn no_room_at_all_does_not_crash() {
    let (_, w) = widget(Book::with(&["ada"]));
    let _h = Harness::new(w, 0.0, 0.0);
}

fn draw(widget: &PeopleWidget, name: &str) -> Option<image::RgbaImage> {
    let dir = std::env::var_os("FS_COMP_SHOTS").map_or_else(
        || std::env::temp_dir().join(format!("fs-people-{}", std::process::id())),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    let canvas = Canvas::new(900, 600, widget.styles());
    match draw_to_png(&canvas, |w, h| widget.build(w, h), || {}, &path) {
        Err(RenderError::NoGraphics(why)) => {
            eprintln!("skipped: {why}");
            None
        }
        other => {
            other.unwrap();
            let picture = image::open(&path).unwrap().to_rgba8();
            if std::env::var_os("FS_COMP_SHOTS").is_none() {
                std::fs::remove_file(&path).unwrap();
            }
            Some(picture)
        }
    }
}

#[test]
fn pictures_of_reading_editing_and_a_failure_differ() {
    let _one = people_common::harness::serial();
    let (book, w) = widget(Book::with(&["ada", "bob"]));
    let Some(read) = draw(&w, "people-read.png") else {
        return;
    };
    w.with_state(|s| s.begin_edit());
    let form = draw(&w, "people-form.png").unwrap();
    assert_ne!(read, form);
    w.with_state(|s| s.set_field("name", "changed"));
    *book.refuse.lock().unwrap() = Some("the store refuses this card".into());
    w.with_state(|s| s.save());
    let failed = draw(&w, "people-failed.png").unwrap();
    assert_ne!(form, failed);
}

#[test]
fn rebuilding_the_workbench_as_a_window_does_keeps_the_reactive_graph_bounded() {
    let (_, w) = widget(Book::numbered(50));
    let mut h = Harness::new(w, 900.0, 600.0);
    let stats = || blinc_core::reactive::global_graph().lock().unwrap().stats();
    for state in ["read", "form"] {
        h.rebuild();
        let before = stats();
        for _ in 0..30 {
            h.rebuild();
        }
        let after = stats();
        eprintln!("{state}: {before:?} -> {after:?}");
        assert!(
            after.derived_count <= before.derived_count + 5,
            "{state}: {before:?} {after:?}"
        );
        assert!(
            after.signal_count <= before.signal_count + 5,
            "{state}: {before:?} {after:?}"
        );
        if state == "read" {
            h.click("edit");
        }
    }
    h.click("cancel");
    h.press("search");
    let before = stats();
    for _ in 0..30 {
        assert!(h.type_text("p"));
        h.frame();
    }
    let after = stats();
    eprintln!("live area: {before:?} -> {after:?}");
    assert!(
        after.derived_count <= before.derived_count + 5,
        "{before:?} {after:?}"
    );
    assert!(
        after.signal_count <= before.signal_count + 5,
        "{before:?} {after:?}"
    );
}
