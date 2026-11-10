//! Moving text and selection between the model and Blinc's text area.
//!
//! Pattern: Adapter — the area counts (line, column); the model counts characters.

use blinc_layout::widgets::text_area::{SharedTextAreaState, TextAreaState};

use crate::model::Selection;

/// Lock without caring about poisoning (a panicked handler must not freeze the editor).
pub(super) fn lock(area: &SharedTextAreaState) -> std::sync::MutexGuard<'_, TextAreaState> {
    area.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The character index of (`line`, `column`).
fn index_of(lines: &[String], line: usize, column: usize) -> usize {
    let before: usize = lines.iter().take(line).map(|l| l.chars().count() + 1).sum();
    let width = lines.get(line).map_or(0, |l| l.chars().count());
    before + column.min(width)
}

/// The (`line`, `column`) of a character index, cut to the lines there are.
fn position_of(lines: &[String], mut index: usize) -> (usize, usize) {
    for (i, line) in lines.iter().enumerate() {
        let width = line.chars().count();
        if index <= width || i + 1 == lines.len() {
            return (i, index.min(width));
        }
        index -= width + 1;
    }
    (0, 0)
}

/// The text of the area and its selection in characters.
pub(super) fn read(area: &SharedTextAreaState) -> (String, Selection) {
    let state = lock(area);
    let at = |p: blinc_layout::widgets::text_area::TextPosition| {
        index_of(&state.lines, p.line, p.column)
    };
    let end = at(state.cursor);
    let start = state.selection_start.map_or(end, at);
    (state.value(), Selection::new(start, end))
}

/// Put `text` and `selection` into the area.
pub(super) fn write(area: &SharedTextAreaState, text: &str, selection: Selection) {
    use blinc_layout::widgets::text_area::TextPosition;
    let mut state = lock(area);
    state.set_value(text);
    let (l0, c0) = position_of(&state.lines, selection.start);
    let (l1, c1) = position_of(&state.lines, selection.end);
    state.cursor = TextPosition::new(l1, c1);
    state.selection_start = (!selection.is_empty()).then(|| TextPosition::new(l0, c0));
}

/// The area drops a final line break; if the model's text has one and the area's has not, it is
/// still there.
pub(super) fn with_final_break(model_text: &str, typed: &str) -> String {
    if model_text.ends_with('\n') && !typed.ends_with('\n') {
        format!("{typed}\n")
    } else {
        typed.to_owned()
    }
}
