//! The checkbox of a table cell is the real checkbox component, the same one as in a list, and it knows the form of the user test
//! (finding E1/E2 of 2026-11-14: `- [ ] Test?` stayed text, and the boxes were drawn as emoji characters).
//!
//! Tests: the real element, laid out with the default look of Blinc's own widgets; an in-process mouse clicks the boxes.

mod common;

use common::serial;
use fs_components_widgets::TextView;
use fs_components_widgets::text::{Block, Inline, ListItem};

fn row(cells: &[&str]) -> Vec<Inline> {
    cells.iter().map(|c| Inline::plain(*c)).collect()
}

/// The cell of the user test: `- [ ] Test?` (a list marker typed in the cell) is a checkbox, like `[ ]` alone.
#[test]
fn the_cell_of_the_user_test_is_a_real_checkbox_the_same_as_in_a_list() {
    use common::{Input, layout_with};
    use std::sync::{Arc, Mutex};
    let _g = serial();
    let clicks: Arc<Mutex<Vec<(usize, bool)>>> = Arc::default();
    let sink = Arc::clone(&clicks);
    let blocks = vec![
        Block::Table {
            align: vec![],
            rows: vec![
                row(&["Name", "Test"]),
                row(&["Kal", "- [ ] Test?"]),
                row(&["Ada", "[x]"]),
            ],
        },
        Block::List {
            ordered: false,
            start: 1,
            items: vec![ListItem::new(vec![Block::Paragraph(Inline::plain("one"))]).task(false)],
        },
    ];
    assert_eq!(
        fs_components_widgets::count_tasks(&blocks),
        3,
        "two cells and one list item"
    );
    let view =
        TextView::new(blocks).on_task(Arc::new(move |i, d| sink.lock().unwrap().push((i, d))), 0);
    let mut tree = layout_with(
        view.build(),
        600.0,
        300.0,
        blinc_layout::ua_styles::UA_STYLES,
    );
    let mut input = Input::new();
    for id in ["fs-tv-task-0", "fs-tv-task-1", "fs-tv-task-2"] {
        assert!(
            tree.query_by_id(id).is_some(),
            "{id}: the same box component in a cell and in a list"
        );
        input.click(&mut tree, id);
    }
    assert_eq!(*clicks.lock().unwrap(), [(0, true), (1, false), (2, true)]);
    // No glyph imitation of a box (the characters U+2610 and U+2611) in any of the places that draw one.
    for (name, src) in [
        ("table.rs", include_str!("../src/text_view/table.rs")),
        ("list.rs", include_str!("../src/text_view/list.rs")),
        ("task.rs", include_str!("../src/text_view/task.rs")),
    ] {
        assert!(
            !src.contains("2610") && !src.contains("2611"),
            "{name} draws a box as a character"
        );
    }
}
