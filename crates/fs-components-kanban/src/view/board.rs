//! The whole board: the bar with "add column" and the notice, then the columns side by side.
//!
//! Pattern: View function — the source is read once per build; the columns are in a box wide
//! enough for all of them, which scrolls, so a column stays readable at any width.

use blinc_layout::div::{Div, div};
use blinc_layout::widgets::scroll::scroll;

use super::column::column;
use super::controls::control;
use crate::handle::KanbanBoard;
use crate::layout::column_width;
use crate::model::Operation;

/// The board filling `width` x `height`.
pub fn board(b: &KanbanBoard, width: f32, height: f32) -> Div {
    let (look, s, pal) = (b.look, b.look.style, b.look.palette());
    let (cols, st) = (b.source.columns(), b.snapshot());
    let gap = s.spacing.s;
    let col_w = column_width(width, cols.len(), gap);
    let inner_w = col_w * cols.len() as f32 + gap * cols.len().saturating_sub(1) as f32;
    let bar_h = s.heights.control + s.spacing.m;
    let add = b.clone();
    let mut bar = div()
        .id("kb-bar")
        .flex_row()
        .items_center()
        .gap_px(s.spacing.s)
        .w_full()
        .h(bar_h)
        .child(control(
            b,
            "kb-add-column",
            "kanban.add_column",
            b.source.supports(Operation::AddColumn),
            false,
            move || add.request(Operation::AddColumn, None),
        ));
    if let Some(code) = &st.notice {
        bar = bar.child(div().id("kb-notice").child(look.text(
            b.say.say(&format!("kanban.refused.{code}")),
            s.text.caption,
            pal.error,
        )));
    }
    let mut row = div()
        .flex_row()
        .gap_px(gap)
        .w(inner_w.max(width))
        .items_start();
    for c in &cols {
        row = row.child(column(b, &st, c, col_w, height - bar_h));
    }
    div().flex_col().w(width).h(height).child(bar).child(
        scroll()
            .both_directions()
            .w(width)
            .h((height - bar_h).max(0.0))
            .content(row),
    )
}
