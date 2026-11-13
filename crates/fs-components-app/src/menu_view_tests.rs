//! Tests of the elements of a menu line: the state it announces and the arrow it shows.
//!
//! Pattern: none — unit tests of the pure line and arrow functions.

use std::sync::Arc;

use crate::menu::MenuEntry;
use crate::menu_view::{Callbacks, Look, arrow, arrows_left, content_width, folds_left, line};

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

#[test]
fn each_line_points_by_the_place_of_its_own_next_level() {
    let narrow = MenuEntry::submenu("a", "A", vec![MenuEntry::new("a1", "X")]);
    let long = "A very long line of a next level that is wider than the widest next level may be, so it is cut at its maximum";
    let wide = MenuEntry::submenu("b", "B", vec![MenuEntry::new("b1", long)]);
    let plain = MenuEntry::new("c", "C");
    let entries = [narrow.clone(), wide.clone(), plain];
    let (w_narrow, w_wide) = (
        content_width(&narrow.children, 600.0),
        content_width(&wide.children, 600.0),
    );
    assert!(w_narrow < w_wide, "{w_narrow} against {w_wide}");
    // Panel at 100, 220 wide, window 600: the narrow level has room on the right, the wide one has not; a line without a level
    // never points anywhere. The widest level must not decide for the others.
    assert_eq!(
        arrows_left(&entries, (100.0, 220.0), 600.0),
        [false, true, false]
    );
    // Counter-probe: in a window with room for both, none folds; and the arrow agrees with the place of each level.
    assert_eq!(
        arrows_left(&entries, (100.0, 220.0), 1200.0),
        [false, false, false]
    );
    for (i, e) in entries.iter().enumerate().take(2) {
        let sub = content_width(&e.children, 600.0);
        assert_eq!(
            arrows_left(&entries, (100.0, 220.0), 600.0)[i],
            folds_left(100.0, 220.0, sub, 600.0)
        );
    }
}
