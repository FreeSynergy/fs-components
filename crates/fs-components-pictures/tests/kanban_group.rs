//! Group `kanban`: a board with three columns, one card with all its parts, and one with a limit.
//!
//! Tests: picture comparison against `contract/umbau/bilder/kanban`.

use std::sync::Arc;

use fs_components_fields::{KeysAsText, Look};
use fs_components_kanban::{
    Card, Column, Due, DueState, FixedBoard, KanbanBoard, Label, Operation, Tone,
};
use fs_components_pictures::{Group, Tolerance};
use fs_components_styles::StyleId;

fn group() -> Group {
    Group::new("kanban", (480, 260), Tolerance::EXACT)
}

fn board(ops: Vec<Operation>) -> KanbanBoard {
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
    let columns = vec![
        todo,
        Column::new("doing", "Doing"),
        Column::new("done", "Done"),
    ];
    KanbanBoard::new(
        Arc::new(FixedBoard { columns, ops }),
        Arc::new(KeysAsText),
        Look::new(StyleId::Craft, false),
    )
}

#[test]
fn a_board_with_moves() {
    let b = board(Operation::ALL.to_vec());
    group().check("board_moves", &fs_components_widgets::styles(), |w, h| {
        b.build(w, h)
    });
}

#[test]
fn a_board_without_operations() {
    let b = board(Vec::new());
    group().check("board_plain", &fs_components_widgets::styles(), |w, h| {
        b.build(w, h)
    });
}
