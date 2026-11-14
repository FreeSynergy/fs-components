//! Tests of the block editor: cutting, putting together, the title field and its Enter.
//!
//! Pattern: none (tests).
//!
//! Tests: model tests over `BlockEditor` (no window).

use super::*;

#[test]
fn blocks_are_cut_at_blank_lines() {
    let got = split_blocks("# T\n\nA\nB\n\n- x\n- y\n");
    assert_eq!(got, ["# T", "A\nB", "- x\n- y"]);
}

#[test]
fn a_fence_is_one_block() {
    let got = split_blocks("a\n\n```rust\nfn x() {}\n\nfn y() {}\n```\n\nb");
    assert_eq!(got, ["a", "```rust\nfn x() {}\n\nfn y() {}\n```", "b"]);
}

#[test]
fn title_and_blocks_go_through_markdown_and_back_unchanged() {
    for text in [
        "# Handbook\n\nIntro\n\n- a\n- b\n",
        "# Handbook\n",
        "# Einleitung\n\nText\n",
        "Text only\n",
        "# Handbook\n\n```\na\n\nb\n```\n\nEnd\n",
        // The heading line is kept as it stood while the title is not changed.
        "# Handbook  \n\nBody\n",
        "#  Handbook\n\nBody\n",
    ] {
        let editor = BlockEditor::new(1, text, "Handbook");
        assert_eq!(editor.text(), text, "round trip of {text:?}");
    }
}

#[test]
fn what_the_round_trip_does_not_keep() {
    // Stated limit: blocks are cut at blank lines, so several blank lines become one.
    let editor = BlockEditor::new(1, "# Handbook\n\n\n\nBody\n", "Handbook");
    assert_eq!(editor.text(), "# Handbook\n\nBody\n");
}

#[test]
fn a_page_with_only_its_title_still_has_a_block_to_type_in() {
    let editor = BlockEditor::new(1, "# Handbook\n", "Handbook");
    assert!(editor.lead_heading());
    assert_eq!(editor.count(), 1, "one empty block");
    assert_eq!(
        editor.text(),
        "# Handbook\n",
        "the empty block is not written"
    );
    editor.area(0).lock().unwrap().set_value("Now text");
    assert_eq!(editor.text(), "# Handbook\n\nNow text\n");
}

#[test]
fn enter_in_the_title_gives_the_keyboard_to_the_first_block() {
    for draft in ["# Handbook\n", "# Handbook\n\nIntro\n"] {
        let editor = BlockEditor::new(1, draft, "Handbook");
        assert!(!editor.area(0).lock().unwrap().is_focused(), "before Enter");
        jump_to(Arc::clone(&editor.blocks))();
        assert!(
            editor.area(0).lock().unwrap().is_focused(),
            "after Enter: {draft:?}"
        );
    }
}

#[test]
fn an_emptied_title_takes_the_heading_out_of_the_text() {
    let editor = BlockEditor::new(1, "# Handbook\n\nIntro\n", "Handbook");
    editor.title.set_text("");
    assert_eq!(editor.text(), "Intro\n");
    editor.title.set_text("   ");
    assert_eq!(editor.text(), "Intro\n");
}

#[test]
fn a_title_with_a_line_break_stays_one_heading_line() {
    let editor = BlockEditor::new(1, "# Handbook\n\nIntro\n", "Handbook");
    editor.title.set_text("Hand\nbook\r\nTwo");
    assert_eq!(editor.text(), "# Hand book  Two\n\nIntro\n");
}

#[test]
fn the_title_is_not_a_block() {
    let editor = BlockEditor::new(1, "# Handbook\n\nIntro\n", "Handbook");
    assert!(editor.lead_heading());
    assert_eq!(editor.count(), 1);
    assert_eq!(editor.title.text(), "Handbook");
    // Counter-test: a first heading that differs from the title stays a block.
    let other = BlockEditor::new(1, "# Einleitung\n\nIntro\n", "Handbook");
    assert!(!other.lead_heading());
    assert_eq!(other.count(), 2);
}

#[test]
fn a_changed_title_is_written_as_the_heading() {
    let editor = BlockEditor::new(1, "# Handbook\n\nIntro\n", "Handbook");
    editor.title.set_text("Guide");
    assert_eq!(editor.text(), "# Guide\n\nIntro\n");
    // A title that was not in the text (a page with another first heading) is added only once it is changed.
    let other = BlockEditor::new(1, "Intro\n", "Handbook");
    assert_eq!(other.text(), "Intro\n");
    other.title.set_text("Guide");
    assert_eq!(other.text(), "# Guide\n\nIntro\n");
}

#[test]
fn join_gives_a_newline_at_the_end() {
    assert_eq!(join_blocks(&["a".into(), "b".into()]), "a\n\nb\n");
}
