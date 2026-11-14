//! Tests of the text of the writing surface: the blocks in it, Enter and Backspace.
//!
//! Pattern: none (tests).
//!
//! Tests: model tests over the pure functions (no window).

use super::surface_text::{after_backspace, after_enter, block_at, block_texts, line_at, spans};

#[test]
fn blocks_are_the_runs_between_blank_lines_and_a_fence_is_one() {
    let text = "# T\n\nA\nB\n\n```\nx\n\ny\n```\n\nend";
    let got: Vec<&str> = spans(text).iter().map(|&(s, e)| &text[s..e]).collect();
    assert_eq!(got, ["# T", "A\nB", "```\nx\n\ny\n```", "end"]);
    assert_eq!(block_texts("a\n\nb"), ["a", "b"]);
    // A gap at the end is the empty block the person is about to write; an empty text has one empty block.
    assert_eq!(block_texts("a\n\n"), ["a", ""]);
    // Enter at the end of a paragraph in the middle of a page: one empty block between the two, and the cursor is in it.
    assert_eq!(block_texts("a\n\n\n\nb"), ["a", "", "b"]);
    assert_eq!(block_at("a\n\n\n\nb", 3), 1);
    assert_eq!(block_at("a\n\n\n\nb", 5), 2);
    assert_eq!(block_texts(""), [""]);
    assert_eq!(block_at("a\n\nb", 0), 0);
    assert_eq!(block_at("a\n\nb", 3), 1);
    assert_eq!(
        block_at("a\n\n", 3),
        1,
        "the cursor in the gap is in the empty block after a"
    );
}

#[test]
fn a_line_is_found_by_a_byte_in_it() {
    assert_eq!(line_at("ab\ncd\nef", 4), (3, 5));
    assert_eq!(line_at("ab", 2), (0, 2));
}

#[test]
fn enter_in_a_paragraph_makes_the_next_block_with_a_gap() {
    assert_eq!(
        after_enter("first line", 5),
        ("first\n\n line".to_owned(), 7)
    );
    assert_eq!(
        after_enter("first line", 10),
        ("first line\n\n".to_owned(), 12)
    );
    // Between blocks, Enter is a line break (more room, no block).
    assert_eq!(after_enter("a\n\nb", 2), ("a\n\n\nb".to_owned(), 3));
}

#[test]
fn enter_in_a_list_goes_on_and_in_code_and_tables_stays_a_line_break() {
    assert_eq!(after_enter("- a", 3), ("- a\n- ".to_owned(), 6));
    assert_eq!(after_enter("- a\n- ", 6), ("- a\n\n".to_owned(), 5));
    assert_eq!(
        after_enter("```\nfn x() {}", 13),
        ("```\nfn x() {}\n".to_owned(), 14)
    );
    assert_eq!(after_enter("| a |", 5), ("| a |\n".to_owned(), 6));
    // The other blocks stay as they are.
    assert_eq!(after_enter("x\n\n- a\n\ny", 6).0, "x\n\n- a\n- \n\ny");
}

#[test]
fn backspace_at_the_start_of_a_block_joins_it_to_the_one_above() {
    // "ab\n\n|cd": Backspace took one line break; the second one goes too.
    assert_eq!(
        after_backspace("ab\n\ncd", "ab\ncd", 3),
        Some(("abcd".to_owned(), 2))
    );
    // An empty block after a paragraph: the gap goes.
    assert_eq!(
        after_backspace("ab\n\n", "ab\n", 3),
        Some(("ab".to_owned(), 2))
    );
    // A list above is not joined, and a deletion elsewhere is none.
    assert_eq!(after_backspace("- a\n\nb", "- a\nb", 4), None);
    assert_eq!(after_backspace("abc", "ab", 2), None);
}
