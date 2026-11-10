//! Tests of the view: drawing, locked controls, unlocking.
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
use ui::{Input, draw_tree, frame, layout, serial};

#[test]
fn the_board_draws_columns_cards_marks_and_initials_in_every_style() {
    let _g = serial();
    for style in StyleId::ALL {
        let h = host(&Operation::ALL);
        let tree = layout(board(&h, style).build(W, H), W, H);
        for id in [
            "kb-col-todo",
            "kb-col-done",
            "kb-head-doing",
            "kb-card-a",
            "kb-card-b",
            "kb-count-todo",
            "kb-who-a",
            "kb-due-a",
            "kb-add-column",
            "kb-rename-todo",
            "kb-limit-todo",
        ] {
            assert!(tree.query_by_id(id).is_some(), "{style:?}: no {id}");
        }
        assert!(tree.query_by_id("kb-notice").is_none());
        if ui::has_fonts() {
            let img = draw_tree(&tree, W as u32, H as u32, "kanban-board.png");
            assert!(ui::ink(&img, (0, 0, W as u32, H as u32), ui::background(&img)) > 500);
        }
    }
}

#[test]
fn unsupported_operations_are_visible_locked_and_say_soon_and_do_nothing() {
    let _g = serial();
    let h = host(&[]);
    let b = board(&h, StyleId::Admin);
    let mut tree = layout(b.build(W, H), W, H);
    let mut input = Input::new();
    for id in ["kb-add-column", "kb-rename-todo", "kb-limit-doing"] {
        assert!(tree.query_by_id(id).is_some(), "{id} stays visible");
    }
    input.click(&mut tree, "kb-add-column");
    input.click(&mut tree, "kb-rename-todo");
    frame(&mut tree, W, H);
    assert!(
        h.log.lock().unwrap().is_empty(),
        "locked controls do nothing"
    );
    input.click(&mut tree, "kb-card-a");
    frame(&mut tree, W, H);
    input.click(&mut tree, "kb-right-a");
    frame(&mut tree, W, H);
    assert_eq!(
        cards_in(&h, "todo"),
        ["a", "b"],
        "a locked move moves nothing"
    );
    assert!(
        tree.query_by_id("kb-notice").is_none(),
        "a locked button has no handler, so no notice"
    );
    assert_eq!(
        b.grab("a"),
        Step::Refused("locked"),
        "a gesture is refused with a reason"
    );
    frame(&mut tree, W, H);
    assert!(
        tree.query_by_id("kb-notice").is_some(),
        "and the reason is shown"
    );
}

#[test]
fn supported_controls_ask_the_host_and_unlock_by_themselves() {
    let _g = serial();
    let h = host(&[Operation::AddColumn]);
    let b = board(&h, StyleId::Windows);
    let mut tree = layout(b.build(W, H), W, H);
    let mut input = Input::new();
    input.click(&mut tree, "kb-add-column");
    assert_eq!(last(&h), "add");
    input.click(&mut tree, "kb-rename-todo");
    assert_eq!(last(&h), "add", "rename is still locked");
    h.ops.lock().unwrap().push(Operation::Rename);
    b.refresh();
    frame(&mut tree, W, H);
    input.click(&mut tree, "kb-rename-todo");
    assert_eq!(
        last(&h),
        "rename todo",
        "the host now supports it: unlocked without a new build"
    );
}
