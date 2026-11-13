//! Tests of the elements of a menu line: the state it announces and the arrow it shows.
//!
//! Pattern: none — unit tests of the pure line and arrow functions.

use std::sync::Arc;

use crate::menu::MenuEntry;
use crate::menu_view::{Callbacks, Look, arrow, folds_left, line};

fn calls() -> Callbacks {
    Callbacks {
        choose: Arc::new(|_, _| {}),
        sub: Arc::new(|_, _, _| {}),
    }
}

fn has(div: &blinc_layout::div::Div, class: &str) -> bool {
    div.classes().iter().any(|c| &**c == class)
}

#[test]
fn the_line_that_opens_a_next_level_says_expanded_only_while_it_is_open() {
    let parent = MenuEntry::submenu("more", "More", vec![MenuEntry::new("x", "X")]);
    let open = line(
        0,
        2,
        &parent,
        (
            false,
            Look {
                open_sub: Some(2),
                ..Look::default()
            },
        ),
        &calls(),
    );
    assert!(has(&open, "aria-expanded=true") && has(&open, "aria-haspopup=menu"));
    // Counter-probes: closed, another line open, and a line without children never says it.
    let closed = line(0, 2, &parent, (false, Look::default()), &calls());
    assert!(!has(&closed, "aria-expanded=true") && has(&closed, "aria-haspopup=menu"));
    let other = line(
        0,
        2,
        &parent,
        (
            false,
            Look {
                open_sub: Some(1),
                ..Look::default()
            },
        ),
        &calls(),
    );
    assert!(!has(&other, "aria-expanded=true"));
    let plain = line(
        0,
        2,
        &MenuEntry::new("p", "P"),
        (
            false,
            Look {
                open_sub: Some(2),
                ..Look::default()
            },
        ),
        &calls(),
    );
    assert!(!has(&plain, "aria-expanded=true") && !has(&plain, "aria-haspopup=menu"));
}

#[test]
fn the_arrow_points_to_the_side_the_next_level_opens_to() {
    assert_eq!(arrow(false), "\u{25b8}");
    assert_eq!(arrow(true), "\u{25c2}");
    // One rule for the place and the arrow: room on the right (a 220 px panel at 100, 200 px level, 800 px window) or not.
    assert!(!folds_left(100.0, 220.0, 200.0, 800.0));
    assert!(folds_left(300.0, 220.0, 200.0, 640.0));
    assert!(
        !folds_left(300.0, 220.0, 200.0, 0.0),
        "an unknown window folds nothing"
    );
    // Exactly at the margin: 100 + 220 + 200 + 8 = 528.
    assert!(!folds_left(100.0, 220.0, 200.0, 528.0) && folds_left(100.0, 220.0, 200.0, 527.0));
}
