//! Heading, list point and quote on lines.
//!
//! Tests: the pure model, no window.

use fs_components_text_editor::{EditCommand, EditorModel, Selection};

fn run(text: &str, a: usize, b: usize, command: &EditCommand) -> (String, Selection) {
    let mut m = EditorModel::new(text);
    m.select(Selection::new(a, b));
    m.apply(command);
    (m.text().to_owned(), m.selection())
}

#[test]
fn a_heading_goes_on_the_line_of_the_caret_and_the_caret_moves_with_the_text() {
    assert_eq!(
        run("one\ntwo\nthree", 5, 5, &EditCommand::Heading(2)),
        ("one\n## two\nthree".into(), Selection::caret(8))
    );
    assert_eq!(
        run("title", 0, 0, &EditCommand::Heading(1)),
        ("# title".into(), Selection::caret(2))
    );
}

#[test]
fn another_level_replaces_the_first_and_the_same_level_takes_it_off() {
    assert_eq!(run("# a", 3, 3, &EditCommand::Heading(3)).0, "### a");
    assert_eq!(
        run("### a", 5, 5, &EditCommand::Heading(3)),
        ("a".into(), Selection::caret(1))
    );
    assert_eq!(run("## a", 0, 0, &EditCommand::Heading(0)).0, "a");
    assert_eq!(run("a", 1, 1, &EditCommand::Heading(9)).0, "###### a");
    assert_eq!(
        run("#no", 0, 0, &EditCommand::Heading(1)).0,
        "# #no",
        "no space, no heading"
    );
}

#[test]
fn a_selection_over_several_lines_changes_each_line() {
    assert_eq!(
        run("a\nb\nc", 0, 5, &EditCommand::ListItem).0,
        "- a\n- b\n- c"
    );
    assert_eq!(run("a\nb\nc", 2, 3, &EditCommand::Quote).0, "a\n> b\nc");
    assert_eq!(
        run("a\nb\nc", 0, 3, &EditCommand::Heading(1)).0,
        "# a\n# b\nc"
    );
}

#[test]
fn a_selection_ending_after_a_line_break_does_not_reach_the_next_line() {
    assert_eq!(run("a\nb\n", 0, 2, &EditCommand::ListItem).0, "- a\nb\n");
}

#[test]
fn list_and_quote_toggle_and_complete_a_mixed_block() {
    assert_eq!(run("- a\n- b", 0, 7, &EditCommand::ListItem).0, "a\nb");
    assert_eq!(run("- a\nb", 0, 5, &EditCommand::ListItem).0, "- a\n- b");
    assert_eq!(
        run("* a", 0, 0, &EditCommand::ListItem).0,
        "a",
        "other bullets count"
    );
    assert_eq!(run("> a\n> b", 0, 7, &EditCommand::Quote).0, "a\nb");
}

#[test]
fn empty_lines_between_are_left_alone_but_a_lone_empty_line_gets_the_mark() {
    assert_eq!(run("a\n\nb", 0, 4, &EditCommand::ListItem).0, "- a\n\n- b");
    assert_eq!(
        run("", 0, 0, &EditCommand::ListItem),
        ("- ".into(), Selection::caret(2))
    );
    assert_eq!(run("x\n\ny", 2, 2, &EditCommand::Quote).0, "x\n> \ny");
}

#[test]
fn many_byte_characters_in_lines() {
    assert_eq!(
        run("ä\n😀b", 3, 3, &EditCommand::ListItem),
        ("ä\n- 😀b".into(), Selection::caret(5))
    );
    assert_eq!(
        run("日本語", 2, 2, &EditCommand::Heading(2)),
        ("## 日本語".into(), Selection::caret(5))
    );
}

#[test]
fn a_caret_inside_the_old_prefix_stays_next_to_the_text() {
    assert_eq!(
        run("## ab", 1, 1, &EditCommand::Heading(1)).1,
        Selection::caret(1)
    );
    assert_eq!(
        run("- ab", 1, 1, &EditCommand::ListItem).1,
        Selection::caret(0)
    );
}
