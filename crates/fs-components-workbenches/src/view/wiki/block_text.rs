//! The text of a block's area: where the cursor is in bytes, and putting a text in with the cursor where it belongs.
//!
//! Pattern: none — small helpers over the state of a text area.

use std::sync::PoisonError;

use blinc_layout::widgets::text_area::{SharedTextAreaState, TextPosition, refresh_text_area};

/// The byte offset of `cursor` in the lines joined by line breaks.
pub(super) fn cursor_byte(lines: &[String], cursor: TextPosition) -> usize {
    let before: usize = lines.iter().take(cursor.line).map(|l| l.len() + 1).sum();
    let line = lines.get(cursor.line).map_or("", String::as_str);
    let column: usize = line.chars().take(cursor.column).map(char::len_utf8).sum();
    before + column
}

/// Put `text` into the area, the cursor `at` bytes into it (else at the end), and draw the area again.
pub(super) fn set_text(state: &SharedTextAreaState, text: &str, at: Option<usize>) {
    {
        let mut d = state.lock().unwrap_or_else(PoisonError::into_inner);
        // Every line, also an empty last one (`set_value` drops it, and a line break at the end is what Enter makes).
        d.set_value(text);
        d.lines = text.split('\n').map(str::to_owned).collect();
        if let Some(at) = at {
            let before = &text[..at.min(text.len())];
            let line = before.matches('\n').count();
            let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count());
            d.cursor = TextPosition::new(line, column);
        }
        d.compute_visual_lines();
    }
    refresh_text_area(state);
}
