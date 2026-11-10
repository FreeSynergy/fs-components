//! Sizes and small derived values: column width, card height, initials, the limit mark.
//!
//! Pattern: none — pure functions; the view takes its measures from here.

use fs_components_styles::Style;

use crate::model::{Card, Column};

/// A column is not drawn narrower than this, in pixels; the board scrolls sideways instead.
pub const COLUMN_MIN_W: f32 = 240.0;

/// Width of one column when `n` columns share `width` with `gap` between them.
pub fn column_width(width: f32, n: usize, gap: f32) -> f32 {
    let n = n.max(1) as f32;
    ((width - gap * (n - 1.0)) / n).max(COLUMN_MIN_W)
}

/// Height of a card in `style`: padding, title, and the optional lines.
pub fn card_height(card: &Card, style: &Style) -> f32 {
    let (sp, t) = (style.spacing, style.text);
    let mut rows = vec![t.body_line];
    if card.subtitle.is_some() {
        rows.push((t.caption * 1.4).ceil());
    }
    if !card.labels.is_empty() {
        rows.push((t.caption * 1.8).ceil());
    }
    if card.due.is_some() || card.assignee.is_some() {
        rows.push((t.caption * 2.0).ceil());
    }
    let gaps = (rows.len() - 1) as f32 * sp.xs;
    rows.iter().sum::<f32>() + gaps + 2.0 * sp.s
}

/// Up to two capital letters from a name: first letters of the first and the last word.
pub fn initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    let letter = |w: &&str| w.chars().next().map(|c| c.to_uppercase().to_string());
    match words.as_slice() {
        [] => String::new(),
        [one] => letter(one).unwrap_or_default(),
        [first, .., last] => letter(first).unwrap_or_default() + &letter(last).unwrap_or_default(),
    }
}

/// The count text of a column header: `3`, or `3/5` with a limit.
pub fn count_mark(column: &Column) -> String {
    match column.limit {
        Some(l) => format!("{}/{l}", column.cards.len()),
        None => column.cards.len().to_string(),
    }
}

/// The column under pointer position `x` (relative to the board), if any.
pub fn column_at(x: f32, col_w: f32, gap: f32, n: usize) -> Option<usize> {
    if x < 0.0 {
        return None;
    }
    let i = (x / (col_w + gap)) as usize;
    (i < n && x - i as f32 * (col_w + gap) <= col_w).then_some(i)
}
