//! A checkbox in a table cell: our extension to GFM.
//!
//! Pattern: none — a pure function over the text of the cell.
//!
//! GFM has task items (`- [ ] a`) in lists only; in a table cell `[ ]` is plain text there. We show a cell that begins
//! with `[ ]`, `[x]` or `[X]` (alone or followed by a space and more text) as a checkbox. The text stays as written:
//! only the drawing differs, and saving writes the cell back unchanged.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::look::Look;
use crate::text::{Block, Inline, Mark};

/// Called with the number of a checkbox (in document order: list tasks and table cells, row by row, depth first) and the
/// state it should have after the click.
pub type TaskHandler = Arc<dyn Fn(usize, bool) + Send + Sync>;

/// The checkboxes of one view: where the count starts, who hears a click, whether clicks are allowed.
pub(super) struct Tasks {
    next: AtomicUsize,
    handler: Option<TaskHandler>,
}

impl Tasks {
    /// Counting from `first`; `handler` none means the boxes are shown and locked.
    pub fn new(first: usize, handler: Option<TaskHandler>) -> Self {
        Self {
            next: AtomicUsize::new(first),
            handler,
        }
    }
}

/// The box of a task item or cell: its glyph, and when the view has tasks, its number as id and its click.
pub(super) fn marker(look: &Look, done: bool, size: f32) -> Div {
    let glyph = if done { "\u{2611}" } else { "\u{2610}" };
    let mut mark = div()
        .class("fs-tv-marker")
        .child(text(glyph).size(size).no_cursor());
    let Some(tasks) = &look.tasks else {
        return mark;
    };
    let index = tasks.next.fetch_add(1, Ordering::Relaxed);
    mark = mark.id(format!("fs-tv-task-{index}"));
    match &tasks.handler {
        Some(handler) => {
            let handler = Arc::clone(handler);
            mark.class("fs-tv-task-on")
                .on_click(move |_| handler(index, !done))
        }
        None => mark.class("fs-tv-task-locked"),
    }
}

/// How many checkboxes the blocks hold: list tasks and table cells that start with `[ ]` or `[x]`.
pub fn count_tasks(blocks: &[Block]) -> usize {
    blocks
        .iter()
        .map(|b| match b {
            Block::List { items, .. } => items
                .iter()
                .map(|i| usize::from(i.task.is_some()) + count_tasks(&i.blocks))
                .sum(),
            Block::Quote(inner) => count_tasks(inner),
            Block::Table { rows, .. } => rows
                .iter()
                .flatten()
                .filter(|c| task_cell(c).is_some())
                .count(),
            _ => 0,
        })
        .sum()
}

/// `Some((done, the cell after the checkbox))` for a checkbox cell, else `None`. The marks of the rest move along.
pub(super) fn task_cell(cell: &Inline) -> Option<(bool, Inline)> {
    let done = match cell.text.get(..3)? {
        "[ ]" => false,
        "[x]" | "[X]" => true,
        _ => return None,
    };
    let rest = &cell.text[3..];
    if !rest.is_empty() && !rest.starts_with(' ') {
        return None;
    }
    let cut = 3 + usize::from(rest.starts_with(' '));
    let marks = cell
        .marks
        .iter()
        .filter(|m| m.start >= cut)
        .map(|m| Mark::new(m.kind.clone(), m.start - cut, m.end - cut))
        .collect();
    Some((done, Inline::new(&cell.text[cut..], marks)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::MarkKind;

    #[test]
    fn a_checkbox_cell_is_read_with_its_state() {
        assert!(!task_cell(&Inline::plain("[ ]")).unwrap().0);
        assert!(task_cell(&Inline::plain("[x]")).unwrap().0);
        assert!(task_cell(&Inline::plain("[X] done")).unwrap().0);
        let (_, rest) = task_cell(&Inline::plain("[ ] buy milk")).unwrap();
        assert_eq!(rest.text, "buy milk");
    }

    #[test]
    fn other_cells_are_text() {
        for text in ["", "[]", "[ ]x", "[y]", "a [ ]", "[ x]"] {
            assert!(task_cell(&Inline::plain(text)).is_none(), "{text:?}");
        }
    }

    #[test]
    fn marks_after_the_box_move_with_the_text() {
        let cell = Inline::new("[x] bold", vec![Mark::new(MarkKind::Strong, 4, 8)]);
        let (_, rest) = task_cell(&cell).unwrap();
        assert_eq!(rest.marks, [Mark::new(MarkKind::Strong, 0, 4)]);
    }
}
