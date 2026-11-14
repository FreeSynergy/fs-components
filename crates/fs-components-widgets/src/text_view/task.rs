//! A checkbox in a table cell: our extension to GFM.
//!
//! Pattern: none — a pure function over the text of the cell.
//!
//! GFM has task items (`- [ ] a`) in lists only; in a table cell `[ ]` is plain text there. We show a cell that begins
//! with `[ ]`, `[x]` or `[X]` (alone or followed by a space and more text) as a checkbox. The text stays as written:
//! only the drawing differs, and saving writes the cell back unchanged.

use crate::text::{Inline, Mark};

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
