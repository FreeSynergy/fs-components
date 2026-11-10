//! Undo, redo, typing, counters and dirty.
//!
//! Tests: the pure model, no window.

use fs_components_text_editor::{Counts, Edit, EditCommand, EditorModel, Selection};

#[test]
fn undo_and_redo_restore_text_and_selection() {
    let mut m = EditorModel::new("hello");
    m.select(Selection::new(0, 5));
    m.apply(&EditCommand::Bold);
    m.apply(&EditCommand::Heading(1));
    assert_eq!(m.text(), "# **hello**");
    assert!(m.undo());
    assert_eq!(m.text(), "**hello**");
    assert!(m.undo());
    assert_eq!((m.text(), m.selection()), ("hello", Selection::new(0, 5)));
    assert!(!m.undo());
    assert!(m.redo() && m.redo());
    assert_eq!(m.text(), "# **hello**");
    assert!(!m.redo());
}

#[test]
fn a_new_step_ends_the_redo_line() {
    let mut m = EditorModel::new("a");
    m.select(Selection::new(0, 1));
    m.apply(&EditCommand::Bold);
    m.undo();
    assert!(m.can_redo());
    m.apply(&EditCommand::Italic);
    assert!(!m.can_redo());
    assert_eq!(m.text(), "*a*");
}

#[test]
fn a_command_that_changes_nothing_is_no_step() {
    let mut m = EditorModel::new("plain");
    assert!(!m.apply(&EditCommand::Heading(0)));
    assert!(!m.can_undo());
}

#[test]
fn typing_is_a_step_with_the_smallest_change() {
    let mut m = EditorModel::new("grüße 😀");
    assert!(m.type_text("grüße 😀!", Selection::caret(8)));
    assert!(m.type_text("gruße 😀!", Selection::caret(2)));
    assert!(!m.type_text("gruße 😀!", Selection::caret(3)));
    assert_eq!(m.selection(), Selection::caret(3));
    assert!(m.undo());
    assert_eq!(m.text(), "grüße 😀!");
    assert!(m.undo());
    assert_eq!((m.text(), m.selection()), ("grüße 😀", Selection::caret(7)));
}

#[test]
fn edit_between_and_invert_are_exact_for_many_byte_text() {
    let e = Edit::between("aöb😀c", "aöXc", Selection::default(), Selection::default()).unwrap();
    assert_eq!(
        (e.at, e.removed.as_str(), e.inserted.as_str()),
        (2, "b😀", "X")
    );
    assert_eq!(e.apply("aöb😀c"), "aöXc");
    assert_eq!(e.invert().apply("aöXc"), "aöb😀c");
    assert!(Edit::between("same", "same", Selection::default(), Selection::default()).is_none());
    assert_eq!(EditCommand::invert(&e), e.invert());
}

#[test]
fn dirty_compares_with_the_saved_text() {
    let mut m = EditorModel::new("a");
    assert!(!m.is_dirty());
    m.select(Selection::new(0, 1));
    m.apply(&EditCommand::Bold);
    assert!(m.is_dirty());
    m.undo();
    assert!(!m.is_dirty(), "back at the start is clean again");
    m.redo();
    m.mark_saved();
    assert!(!m.is_dirty());
    m.undo();
    assert!(m.is_dirty(), "undo past the save is a change");
    m.set_text("loaded");
    assert!(!m.is_dirty() && !m.can_undo());
}

#[test]
fn counters_count_characters_and_words() {
    assert_eq!(Counts::of(""), Counts { chars: 0, words: 0 });
    assert_eq!(
        Counts::of("ein  Wort\nzwei"),
        Counts {
            chars: 14,
            words: 3
        }
    );
    assert_eq!(
        EditorModel::new("grüße 😀").counts(),
        Counts { chars: 7, words: 2 }
    );
}

#[test]
fn the_history_is_bounded() {
    let mut m = EditorModel::new("");
    for i in 0..(fs_components_text_editor::model::MAX_HISTORY + 20) {
        m.type_text(&"x".repeat(i + 1), Selection::caret(i + 1));
    }
    let mut steps = 0;
    while m.undo() {
        steps += 1;
    }
    assert_eq!(steps, fs_components_text_editor::model::MAX_HISTORY);
}
