//! `ReaderWidget`: what a program that embeds the reader sees, through the public API only.
//!
//! The text area drops a final newline (the reader puts it back when saving).
//!
//! Pattern: State machine tests through the widget (no window needed for the text area logic).

mod common;

use std::sync::Arc;

use common::Editable;
use fs_components_workbenches::{Labels, Reader, ReaderWidget};

fn widget() -> (Arc<Editable>, ReaderWidget) {
    let source = Arc::new(Editable::with(&[
        ("a", "text of A\n"),
        ("b", "text of B\n"),
    ]));
    let widget = ReaderWidget::new(source.clone(), Labels::default(), None);
    (source, widget)
}

#[test]
fn no_edit_no_text_area() {
    let (_s, w) = widget();
    assert_eq!(w.editor_text(), None);
}

#[test]
fn an_edit_of_another_document_never_shows_the_text_of_the_first() {
    let (source, w) = widget();
    w.with_reader(Reader::begin_edit);
    assert_eq!(w.editor_text().as_deref(), Some("text of A"));
    // A program's command: go to b and edit it, without the first edit having been touched.
    w.with_reader(|r| {
        r.open("b");
        r.begin_edit();
    });
    assert_eq!(w.editor_text().as_deref(), Some("text of B"));
    // The person types one character and saves: B must become B's text plus the character.
    w.with_reader(|r| r.set_draft("text of B!"));
    w.with_reader(Reader::save);
    assert_eq!(source.text("b"), "text of B!\n");
    assert_eq!(source.text("a"), "text of A\n", "A must stay as it was");
}

#[test]
fn a_draft_set_from_outside_reaches_the_text_area() {
    let (_s, w) = widget();
    w.with_reader(Reader::begin_edit);
    w.with_reader(|r| r.set_draft("changed by a command"));
    assert_eq!(w.editor_text().as_deref(), Some("changed by a command"));
}

#[test]
fn the_area_is_dropped_when_the_edit_ends() {
    let (_s, w) = widget();
    w.with_reader(Reader::begin_edit);
    assert!(w.editor_text().is_some());
    w.with_reader(Reader::cancel_edit);
    assert_eq!(w.editor_text(), None);
}
