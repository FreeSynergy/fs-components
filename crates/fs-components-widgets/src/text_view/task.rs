//! A checkbox in a table cell: our extension to GFM.
//!
//! Pattern: none — a pure function over the text of the cell.
//!
//! GFM has task items (`- [ ] a`) in lists only; in a table cell `[ ]` is plain text there. We show a cell that begins
//! with `[ ]`, `[x]` or `[X]` (alone or followed by a space and more text) as a checkbox. The text stays as written:
//! only the drawing differs, and saving writes the cell back unchanged.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use blinc_cn::components::checkbox::{CheckboxBuilder, CheckboxSize};
use blinc_core::reactive::signal;
use blinc_layout::div::{Div, div};

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

/// The box of a task item or cell: the real checkbox component of the theme (`cn::checkbox`), the same one in a list and in
/// a table cell. When the view has tasks it carries its number as id and reports a click with the state it got; without a
/// handler it is locked: its state is drawn, a click is put back.
pub(super) fn marker(look: &Look, done: bool, _size: f32) -> Div {
    static KEYS: AtomicUsize = AtomicUsize::new(0);
    let state = signal(done);
    let key = format!("fs-tv-box-{}", KEYS.fetch_add(1, Ordering::Relaxed));
    let mut box_ = CheckboxBuilder::with_key(key, state).size(CheckboxSize::Small);
    let mut id = None;
    let handler = look.tasks.as_ref().and_then(|t| t.handler.clone());
    if let Some(tasks) = &look.tasks {
        let index = tasks.next.fetch_add(1, Ordering::Relaxed);
        id = Some(format!("fs-tv-task-{index}"));
    }
    let index = id
        .as_deref()
        .and_then(|i| i.strip_prefix("fs-tv-task-"))
        .and_then(|n| n.parse::<usize>().ok());
    match (handler, index) {
        (Some(handler), Some(index)) => {
            box_ = box_.on_change(move |now| handler(index, now));
        }
        // Locked: the box is not disabled (a disabled box does not show that it is ticked), a click is put back.
        _ => box_ = box_.on_change(move |now| state.set(!now)),
    }
    let mut component = box_.build_component();
    if let Some(id) = &id {
        component = component.id(id);
    }
    let mark = div().class("fs-tv-marker").child(component);
    if look.tasks.as_ref().is_some_and(|t| t.handler.is_none()) || look.tasks.is_none() {
        mark.cursor_not_allowed()
    } else {
        mark
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

/// The box a cell starts with: (bytes before the bracket, bytes up to and with the space after the box, the state).
/// The forms are `[ ]`, `[x]`, `[X]` and, as typed in a list, `- [ ]`, `- [x]`; the box is alone or followed by a space.
pub fn cell_box(cell: &str) -> Option<(usize, usize, bool)> {
    let lead = if cell.starts_with("- ") { 2 } else { 0 };
    let done = match cell.get(lead..lead + 3)? {
        "[ ]" => false,
        "[x]" | "[X]" => true,
        _ => return None,
    };
    let rest = &cell[lead + 3..];
    if !rest.is_empty() && !rest.starts_with(' ') {
        return None;
    }
    Some((lead, lead + 3 + usize::from(rest.starts_with(' ')), done))
}

/// `Some((done, the cell after the checkbox))` for a checkbox cell, else `None`. The marks of the rest move along.
pub(super) fn task_cell(cell: &Inline) -> Option<(bool, Inline)> {
    let (_, cut, done) = cell_box(&cell.text)?;
    let char_cut = cell.text[..cut].chars().count();
    let marks = cell
        .marks
        .iter()
        .filter(|m| m.start >= char_cut)
        .map(|m| Mark::new(m.kind.clone(), m.start - char_cut, m.end - char_cut))
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
        // The form of the user test: a list marker in the cell.
        assert!(!task_cell(&Inline::plain("- [ ] Test?")).unwrap().0);
        assert!(task_cell(&Inline::plain("- [x]")).unwrap().0);
        assert_eq!(
            task_cell(&Inline::plain("- [ ] Test?")).unwrap().1.text,
            "Test?"
        );
        let (_, rest) = task_cell(&Inline::plain("[ ] buy milk")).unwrap();
        assert_eq!(rest.text, "buy milk");
    }

    #[test]
    fn other_cells_are_text() {
        for text in [
            "", "[]", "[ ]x", "[y]", "a [ ]", "[ x]", "-[ ]", "- [ ]x", "* [ ]",
        ] {
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
