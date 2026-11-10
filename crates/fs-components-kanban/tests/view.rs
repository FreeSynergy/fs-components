//! Tests of the view: drawn offscreen and driven by clicks and keys, as a window would.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::{Arc, Mutex};

use fs_components_fields::{KeysAsText, Look};
use fs_components_kanban::*;
use fs_components_styles::StyleId;
use ui::{Input, draw_tree, frame, layout, serial};

const W: f32 = 900.0;
const H: f32 = 600.0;

/// A host: keeps the columns, answers moves, remembers requests.
struct Host {
    cols: Mutex<Vec<Column>>,
    ops: Mutex<Vec<Operation>>,
    log: Mutex<Vec<String>>,
}

impl BoardSource for Host {
    fn columns(&self) -> Vec<Column> {
        self.cols.lock().unwrap().clone()
    }
    fn supports(&self, op: Operation) -> bool {
        self.ops.lock().unwrap().contains(&op)
    }
}

impl BoardEvents for Host {
    fn select_card(&self, key: &str) {
        self.log.lock().unwrap().push(format!("select {key}"));
    }
    fn move_card(&self, key: &str, from: &str, to: &str) -> Answer {
        if to == "done" && key == "b" {
            return Answer::Refused("not_allowed".into());
        }
        let mut cols = self.cols.lock().unwrap();
        let card = {
            let f = cols.iter_mut().find(|c| c.key == from).unwrap();
            let i = f.cards.iter().position(|c| c.key == key).unwrap();
            f.cards.remove(i)
        };
        cols.iter_mut()
            .find(|c| c.key == to)
            .unwrap()
            .cards
            .push(card);
        self.log
            .lock()
            .unwrap()
            .push(format!("move {key} {from} {to}"));
        Answer::Accepted
    }
    fn add_column(&self) {
        self.log.lock().unwrap().push("add".into());
    }
    fn rename_column(&self, c: &str) {
        self.log.lock().unwrap().push(format!("rename {c}"));
    }
}

fn host(ops: &[Operation]) -> Arc<Host> {
    let mut a = Card::new("a", "Write the spec");
    a.subtitle = Some("Part one".into());
    a.labels = vec![Label {
        text: "docs".into(),
        tone: Tone::Success,
    }];
    a.due = Some(Due {
        text: "12 Nov".into(),
        state: DueState::Overdue,
    });
    a.assignee = Some("Ada Lovelace".into());
    let mut todo = Column::new("todo", "To do");
    todo.cards = vec![a, Card::new("b", "Review")];
    todo.limit = Some(1);
    Arc::new(Host {
        cols: Mutex::new(vec![
            todo,
            Column::new("doing", "Doing"),
            Column::new("done", "Done"),
        ]),
        ops: Mutex::new(ops.to_vec()),
        log: Mutex::default(),
    })
}

fn board(h: &Arc<Host>, style: StyleId) -> KanbanBoard {
    KanbanBoard::new(h.clone(), Arc::new(KeysAsText), Look::new(style, false)).events(h.clone())
}

fn last(h: &Host) -> String {
    h.log.lock().unwrap().last().cloned().unwrap_or_default()
}

fn cards_in(h: &Host, col: &str) -> Vec<String> {
    h.columns()
        .iter()
        .find(|c| c.key == col)
        .unwrap()
        .cards
        .iter()
        .map(|c| c.key.clone())
        .collect()
}

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
