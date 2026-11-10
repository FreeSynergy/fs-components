//! The two boundaries to the host: where the columns come from and who hears of requests.
//!
//! Pattern: Strategy (`BoardSource`, `BoardEvents`) with a Null object (`NoEvents`). The board asks
//! the source on every build, so the host stays the one owner of its data; every change of the
//! board is a request to the host, which answers or refuses it. The board holds no text input: a
//! rename or a limit is asked of the reader by the host (a dialog or form of its own).

use crate::model::{Column, Operation};

/// Where the columns and the supported operations come from.
pub trait BoardSource: Send + Sync {
    /// The columns with their cards, in order.
    fn columns(&self) -> Vec<Column>;
    /// Whether the host can do `op` now. A control of an unsupported operation is drawn, locked,
    /// with the "comes later" hint, and unlocks by itself when this turns true.
    fn supports(&self, op: Operation) -> bool;
}

/// What the host says to a move.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// Done; the next `columns()` shows it.
    Accepted,
    /// Refused; the code is the tail of the translation key `kanban.refused.<code>`.
    Refused(String),
}

/// What the board tells the host. Every method has a default that does nothing.
pub trait BoardEvents: Send + Sync {
    /// A card was chosen (click, or Enter).
    fn select_card(&self, _key: &str) {}
    /// A card should move to column `to`.
    fn move_card(&self, _key: &str, _from: &str, _to: &str) -> Answer {
        Answer::Refused("unsupported".into())
    }
    /// The reader wants a new column.
    fn add_column(&self) {}
    /// The reader wants to rename a column.
    fn rename_column(&self, _column: &str) {}
    /// The reader wants to set the limit of a column.
    fn set_limit(&self, _column: &str) {}
}

/// Hears nothing; refuses every move.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoEvents;

impl BoardEvents for NoEvents {}

/// A fixed board for tests and demos.
#[derive(Clone, Debug)]
pub struct FixedBoard {
    /// The columns.
    pub columns: Vec<Column>,
    /// The supported operations.
    pub ops: Vec<Operation>,
}

impl BoardSource for FixedBoard {
    fn columns(&self) -> Vec<Column> {
        self.columns.clone()
    }
    fn supports(&self, op: Operation) -> bool {
        self.ops.contains(&op)
    }
}
