//! The view: it is built, the buttons run the commands, typing and the listener, undo and redo.
//!
//! Tests: the window's own router and hit test, fed in-process (see the widgets' `tests/common`).

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use std::sync::{Arc, Mutex};

use blinc_layout::widgets::text_area::TextPosition;
use common::{Input, layout, serial};
use fs_components_text_editor::{Change, EditorView, Labels, Selection, TextEditor};
use fs_components_widgets::{Block, Inline};

type Heard = Arc<Mutex<Vec<Change>>>;

fn editor() -> (EditorView, Heard) {
    let heard: Heard = Arc::default();
    let sink = heard.clone();
    let view = EditorView::new(Arc::new(|text| vec![Block::Paragraph(Inline::plain(text))]))
        .on_change(Arc::new(move |c| sink.lock().unwrap().push(c.clone())));
    (view, heard)
}

/// What the person does in the text area: select from `a` to `b` in line 0.
fn select_in_line(view: &EditorView, a: usize, b: usize) {
    let mut area = view.area().lock().unwrap();
    area.selection_start = Some(TextPosition::new(0, a));
    area.cursor = TextPosition::new(0, b);
}

fn area_text(view: &EditorView) -> String {
    view.area().lock().unwrap().value()
}

#[test]
fn it_is_built_with_all_its_buttons_and_the_counters() {
    let _g = serial();
    let (mut view, _) = editor();
    view.set_text("grüße 😀");
    let tree = layout(view.build(), 600.0, 500.0);
    for slot in [
        "bold", "italic", "h1", "h2", "h3", "list", "link", "code", "quote", "undo", "redo",
        "counts",
    ] {
        assert!(
            tree.query_by_id(&format!("fs-te-{slot}")).is_some(),
            "no {slot}"
        );
    }
    assert!(tree.query_by_id("fs-te-unsaved").is_none());
}

#[test]
fn a_click_on_bold_wraps_what_is_selected_and_the_listener_hears_it() {
    let _g = serial();
    let (mut view, heard) = editor();
    view.set_text("hello world");
    select_in_line(&view, 0, 5);
    let mut tree = layout(view.build(), 600.0, 500.0);
    Input::new().click(&mut tree, "fs-te-bold");
    assert_eq!(view.text(), "**hello** world");
    assert_eq!(area_text(&view), "**hello** world");
    assert_eq!(view.selection(), Selection::new(2, 7));
    let area = view.area().lock().unwrap();
    assert_eq!(
        (area.selection_start, area.cursor),
        (Some(TextPosition::new(0, 2)), TextPosition::new(0, 7))
    );
    let last = heard.lock().unwrap().last().cloned().unwrap();
    assert_eq!(last.text, "**hello** world");
    assert!(last.dirty);
}

#[test]
fn every_button_runs_its_command() {
    let _g = serial();
    let cases = [
        ("italic", "*word*"),
        ("h1", "# word"),
        ("h2", "## word"),
        ("h3", "### word"),
        ("list", "- word"),
        ("link", "[word]()"),
        ("code", "`word`"),
        ("quote", "> word"),
    ];
    for (slot, expected) in cases {
        let (mut view, _) = editor();
        view.set_text("word");
        select_in_line(&view, 0, 4);
        let mut tree = layout(view.build(), 600.0, 500.0);
        Input::new().click(&mut tree, &format!("fs-te-{slot}"));
        assert_eq!(view.text(), expected, "{slot}");
    }
}

#[test]
fn undo_and_redo_buttons_walk_the_history() {
    let _g = serial();
    let (mut view, heard) = editor();
    view.set_text("x");
    select_in_line(&view, 0, 1);
    let mut tree = layout(view.build(), 600.0, 500.0);
    let mut input = Input::new();
    input.click(&mut tree, "fs-te-bold");
    assert_eq!(view.text(), "**x**");
    let mut tree = layout(view.build(), 600.0, 500.0);
    input.click(&mut tree, "fs-te-undo");
    assert_eq!(
        (view.text().as_str(), area_text(&view).as_str()),
        ("x", "x")
    );
    assert!(!heard.lock().unwrap().last().unwrap().dirty);
    let mut tree = layout(view.build(), 600.0, 500.0);
    input.click(&mut tree, "fs-te-redo");
    assert_eq!(view.text(), "**x**");
}

#[test]
fn typing_is_taken_over_and_reported_and_can_be_undone() {
    let _g = serial();
    let (mut view, heard) = editor();
    view.set_text("ab");
    {
        let mut area = view.area().lock().unwrap();
        area.set_value("abc");
    }
    view.typed("abc");
    assert_eq!(view.text(), "abc");
    assert!(view.is_dirty());
    assert_eq!(heard.lock().unwrap().last().unwrap().counts.chars, 3);
    assert!(view.undo());
    assert_eq!(area_text(&view), "ab");
    view.mark_saved();
    assert!(!view.is_dirty());
    assert!(!heard.lock().unwrap().last().unwrap().dirty);
}

#[test]
fn a_click_on_undo_without_history_changes_nothing() {
    let _g = serial();
    let (view, _) = editor();
    let mut labels = Labels::default();
    labels.bold = "Fett".into();
    let view = view.labels(labels).id("ed");
    assert!(!view.can_undo() && !view.can_redo());
    let mut tree = layout(view.build(), 600.0, 500.0);
    Input::new().click(&mut tree, "ed-undo");
    assert_eq!(view.text(), "");
}
