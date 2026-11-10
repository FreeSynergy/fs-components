//! Test support: a host with columns, answers and a log.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic, dead_code)]

use std::sync::{Arc, Mutex};

use fs_components_fields::{KeysAsText, Look};
use fs_components_kanban::*;
use fs_components_styles::StyleId;

pub const W: f32 = 900.0;
pub const H: f32 = 600.0;

/// A host: keeps the columns, answers moves, remembers requests.
pub struct Host {
    pub cols: Mutex<Vec<Column>>,
    pub ops: Mutex<Vec<Operation>>,
    pub log: Mutex<Vec<String>>,
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

pub fn host(ops: &[Operation]) -> Arc<Host> {
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

pub fn board(h: &Arc<Host>, style: StyleId) -> KanbanBoard {
    KanbanBoard::new(h.clone(), Arc::new(KeysAsText), Look::new(style, false)).events(h.clone())
}

pub fn last(h: &Host) -> String {
    h.log.lock().unwrap().last().cloned().unwrap_or_default()
}

pub fn cards_in(h: &Host, col: &str) -> Vec<String> {
    h.columns()
        .iter()
        .find(|c| c.key == col)
        .unwrap()
        .cards
        .iter()
        .map(|c| c.key.clone())
        .collect()
}
