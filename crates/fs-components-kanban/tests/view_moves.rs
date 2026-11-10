//! Tests of the view: moves by buttons, gesture API, click and keyboard.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_kanban::*;
use fs_components_styles::StyleId;
use host::{H, W, board, cards_in, host, last};
use ui::{Input, frame, layout, serial};

#[test]
fn move_buttons_move_a_card_and_a_refusal_is_shown() {
    let _g = serial();
    let h = host(&Operation::ALL);
    let b = board(&h, StyleId::Apple);
    let mut tree = layout(b.build(W, H), W, H);
    let mut input = Input::new();
    input.click(&mut tree, "kb-card-a");
    frame(&mut tree, W, H);
    assert_eq!(last(&h), "select a");
    input.click(&mut tree, "kb-right-a");
    frame(&mut tree, W, H);
    assert_eq!(cards_in(&h, "doing"), ["a"]);
    assert_eq!(b.snapshot().notice, None);
    input.click(&mut tree, "kb-card-b");
    frame(&mut tree, W, H);
    input.click(&mut tree, "kb-right-b");
    frame(&mut tree, W, H);
    input.click(&mut tree, "kb-right-b");
    frame(&mut tree, W, H);
    assert_eq!(
        cards_in(&h, "done"),
        Vec::<String>::new(),
        "refused by the host"
    );
    assert!(tree.query_by_id("kb-notice").is_some());
}

#[test]
fn grab_hover_drop_by_api_and_click_on_a_column() {
    let _g = serial();
    let h = host(&Operation::ALL);
    let b = board(&h, StyleId::Admin);
    let mut tree = layout(b.build(W, H), W, H);
    let mut input = Input::new();
    b.grab("a");
    b.hover("doing");
    frame(&mut tree, W, H);
    assert_eq!(b.snapshot().held.unwrap().over, "doing");
    b.drop_held();
    frame(&mut tree, W, H);
    assert_eq!(last(&h), "move a todo doing");
    // click-to-move: pick up, then click the target column
    b.grab("b");
    frame(&mut tree, W, H);
    let b0 = tree
        .get_absolute_bounds(tree.query_by_id("kb-col-doing").unwrap())
        .unwrap();
    input.click_at(&mut tree, b0.x + b0.width / 2.0, b0.y + b0.height - 10.0);
    frame(&mut tree, W, H);
    assert_eq!(cards_in(&h, "doing"), ["a", "b"]);
    assert!(b.snapshot().held.is_none());
}

#[test]
fn the_keyboard_picks_up_moves_and_puts_down() {
    let _g = serial();
    let h = host(&Operation::ALL);
    let b = board(&h, StyleId::Simple);
    let mut tree = layout(b.build(W, H), W, H);
    let mut input = Input::new();
    input.focus(&tree, "kb-root");
    input.key(&mut tree, 40); // down: cursor on the first card
    frame(&mut tree, W, H);
    input.key(&mut tree, 13); // enter on the root is a click: Activate -> grab
    frame(&mut tree, W, H);
    assert_eq!(b.snapshot().held.unwrap().key, "a");
    input.key(&mut tree, 39); // right
    frame(&mut tree, W, H);
    input.key(&mut tree, 13); // activate: drop
    frame(&mut tree, W, H);
    assert_eq!(last(&h), "move a todo doing");
    input.key(&mut tree, 40);
    input.key(&mut tree, 13);
    input.key(&mut tree, 27); // escape
    frame(&mut tree, W, H);
    assert!(b.snapshot().held.is_none());
}
