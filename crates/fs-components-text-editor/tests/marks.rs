//! Bold, italic and code: wrap, take off, caret only, many-byte characters.
//!
//! Tests: the pure model, no window.

use fs_components_text_editor::{EditCommand, EditorModel, Selection, TextEditor};

fn run(text: &str, a: usize, b: usize, command: &EditCommand) -> (String, Selection) {
    let mut m = EditorModel::new(text);
    m.select(Selection::new(a, b));
    m.apply(command);
    (m.text().to_owned(), m.selection())
}

#[test]
fn a_selection_is_wrapped_and_stays_selected() {
    assert_eq!(
        run("hello world", 0, 5, &EditCommand::Bold),
        ("**hello** world".into(), Selection::new(2, 7))
    );
    assert_eq!(
        run("hello world", 6, 11, &EditCommand::Italic),
        ("hello *world*".into(), Selection::new(7, 12))
    );
    assert_eq!(
        run("a b", 2, 3, &EditCommand::Code),
        ("a `b`".into(), Selection::new(3, 4))
    );
}

#[test]
fn wrapping_twice_takes_the_mark_off_again() {
    for command in [EditCommand::Bold, EditCommand::Italic, EditCommand::Code] {
        let mut m = EditorModel::new("word");
        m.select(Selection::new(0, 4));
        m.apply(&command);
        m.apply(&command);
        assert_eq!(m.text(), "word", "{command:?}");
        assert_eq!(m.selection(), Selection::new(0, 4));
    }
}

#[test]
fn a_selection_that_includes_the_marks_loses_them() {
    assert_eq!(
        run("a **b** c", 2, 7, &EditCommand::Bold),
        ("a b c".into(), Selection::new(2, 3))
    );
    assert_eq!(
        run("*x*", 0, 3, &EditCommand::Italic),
        ("x".into(), Selection::new(0, 1))
    );
}

#[test]
fn italic_does_not_mistake_bold_for_itself() {
    assert_eq!(run("**x**", 2, 3, &EditCommand::Italic).0, "***x***");
    assert_eq!(run("***x***", 3, 4, &EditCommand::Italic).0, "**x**");
    assert_eq!(run("***x***", 3, 4, &EditCommand::Bold).0, "*x*");
    assert_eq!(run("*x*", 1, 2, &EditCommand::Bold).0, "***x***");
}

#[test]
fn no_selection_makes_an_empty_pair_with_the_caret_inside() {
    assert_eq!(
        run("ab", 1, 1, &EditCommand::Bold),
        ("a****b".into(), Selection::caret(3))
    );
    assert_eq!(
        run("", 0, 0, &EditCommand::Italic),
        ("**".into(), Selection::caret(1))
    );
    assert_eq!(
        run("x", 1, 1, &EditCommand::Code),
        ("x``".into(), Selection::caret(2))
    );
}

#[test]
fn the_empty_pair_is_removed_by_the_same_button() {
    let mut m = EditorModel::new("ab");
    m.select(Selection::caret(1));
    m.apply(&EditCommand::Bold);
    m.apply(&EditCommand::Bold);
    assert_eq!((m.text(), m.selection()), ("ab", Selection::caret(1)));
}

#[test]
fn positions_count_characters_not_bytes() {
    // 'ä' is two bytes, the emoji four, the e with a combining accent two characters
    assert_eq!(
        run("äöü 😀 end", 0, 3, &EditCommand::Bold),
        ("**äöü** 😀 end".into(), Selection::new(2, 5))
    );
    assert_eq!(
        run("äöü 😀 end", 4, 5, &EditCommand::Code),
        ("äöü `😀` end".into(), Selection::new(5, 6))
    );
    assert_eq!(
        run("e\u{301}x", 0, 2, &EditCommand::Italic).0,
        "*e\u{301}*x"
    );
    assert_eq!(
        run("😀", 1, 1, &EditCommand::Bold),
        ("😀****".into(), Selection::caret(3))
    );
}

#[test]
fn a_selection_past_the_end_is_cut_to_the_text() {
    assert_eq!(run("ab", 1, 99, &EditCommand::Bold).0, "a**b**");
}

#[test]
fn a_link_wraps_the_selection_or_makes_an_empty_one() {
    let link = EditCommand::Link("https://example.org".into());
    assert_eq!(
        run("see here now", 4, 8, &link),
        (
            "see [here](https://example.org) now".into(),
            Selection::new(5, 9)
        )
    );
    assert_eq!(
        run("ab", 1, 1, &link),
        ("a[](https://example.org)b".into(), Selection::caret(2))
    );
    assert_eq!(run("ö", 0, 1, &EditCommand::Link(String::new())).0, "[ö]()");
}

#[test]
fn the_editor_trait_runs_the_same_commands() {
    fn through(editor: &mut dyn TextEditor) -> String {
        editor.set_text("x");
        editor.select(Selection::new(0, 1));
        assert!(editor.apply(&EditCommand::Bold));
        editor.text()
    }
    assert_eq!(through(&mut EditorModel::default()), "**x**");
}
