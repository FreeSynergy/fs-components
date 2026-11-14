//! Tests of what Enter does in a block.
//!
//! Pattern: none (tests).
//!
//! Tests: model tests over the pure function `enter` (no window).

use super::block_enter::{Enter, enter};

/// What `enter` says for `before|after` (the line break is added at the bar).
fn at(before: &str, after: &str) -> Enter {
    let now = format!("{before}\n{after}");
    enter(&now, before.len() + 1)
}

#[test]
fn a_paragraph_splits_at_the_cursor() {
    assert_eq!(
        at("first", "second"),
        Enter::Split {
            head: "first".into(),
            tail: "second".into()
        }
    );
    assert_eq!(
        at("first", ""),
        Enter::Split {
            head: "first".into(),
            tail: String::new()
        }
    );
}

#[test]
fn a_list_carries_on_with_the_next_marker() {
    assert_eq!(
        at("- a", ""),
        Enter::Carry {
            text: "- a\n- ".into(),
            at: 6
        }
    );
    assert_eq!(
        at("1. a\n2. b", ""),
        Enter::Carry {
            text: "1. a\n2. b\n3. ".into(),
            at: 13
        }
    );
    assert_eq!(
        at("- [x] done", ""),
        Enter::Carry {
            text: "- [x] done\n- [ ] ".into(),
            at: 17
        }
    );
}

#[test]
fn an_empty_item_ends_the_list() {
    assert_eq!(
        at("- a\n- ", ""),
        Enter::Split {
            head: "- a".into(),
            tail: String::new()
        }
    );
}

#[test]
fn code_and_tables_keep_their_line_break() {
    assert_eq!(at("```rust\nfn x() {}", ""), Enter::Keep);
    assert_eq!(at("| a | b |", ""), Enter::Keep);
    // A closed fence is a paragraph again.
    assert!(matches!(at("```\nx\n```", ""), Enter::Split { .. }));
}
