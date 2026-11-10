//! One column: header (title, count, limit mark, controls) and its cards.
//!
//! Pattern: View function. A held card over a column marks it as the target; a click on a column
//! while a card is held puts the card down there (the drag-free move).

use blinc_layout::div::{Div, div};
use fs_components_fields::Look;
use fs_components_styles::Glyph;

use super::card::card;
use super::controls::control;
use crate::handle::KanbanBoard;
use crate::layout::count_mark;
use crate::model::{Column, Operation};
use crate::state::BoardState;

/// The column `col`, `width` pixels wide and at least `min_h` high.
pub fn column(b: &KanbanBoard, st: &BoardState, col: &Column, width: f32, min_h: f32) -> Div {
    let (look, s, pal) = (b.look, b.look.style, b.look.palette());
    let target = st.held.as_ref().is_some_and(|h| h.over == col.key);
    let (enter, click, key, ck) = (b.clone(), b.clone(), col.key.clone(), col.key.clone());
    let (k1, k2) = (key.clone(), key.clone());
    let inner = width - 2.0 * s.spacing.s;
    let over = col.over_limit();
    let mark_color = if over { pal.error } else { pal.text_weak };
    let mut mark = div()
        .id(format!("kb-count-{key}"))
        .flex_row()
        .items_center()
        .gap_px(s.spacing.xs);
    if over {
        mark = mark.child(look.icon(Glyph::Warning, Some(s.text.caption), pal.error));
    }
    let mark = mark.child(look.text(count_mark(col), s.text.caption, mark_color));
    let (rename_b, limit_b) = (b.clone(), b.clone());
    let controls = div()
        .flex_row()
        .gap_px(s.spacing.xs)
        .child(control(
            b,
            &format!("kb-rename-{key}"),
            "kanban.rename",
            b.source.supports(Operation::Rename),
            true,
            move || rename_b.request(Operation::Rename, Some(&k1)),
        ))
        .child(control(
            b,
            &format!("kb-limit-{key}"),
            "kanban.limit",
            b.source.supports(Operation::Limit),
            true,
            move || limit_b.request(Operation::Limit, Some(&k2)),
        ));
    let head = div()
        .id(format!("kb-head-{key}"))
        .flex_col()
        .gap_px(s.spacing.xs)
        .child(
            div()
                .flex_row()
                .items_center()
                .gap_px(s.spacing.s)
                .child(div().flex_grow_value(1.0).child(look.text(
                    &col.title,
                    s.text.title,
                    pal.text,
                )))
                .child(mark),
        )
        .child(controls);
    let mut body = div().flex_col().gap_px(s.spacing.s).w(inner);
    for c in &col.cards {
        body = body.child(card(b, st, c, &col.key, inner));
    }
    if col.cards.is_empty() {
        body = body.child(look.text(b.say.say("kanban.empty"), s.text.caption, pal.text_weak));
    }
    div()
        .id(format!("kb-col-{}", col.key))
        .flex_col()
        .gap_px(s.spacing.s)
        .w(width)
        .min_h(min_h)
        .padding_x_px(s.spacing.s)
        .padding_y_px(s.spacing.s)
        .rounded(s.radii.card)
        .bg(Look::color(pal.ground))
        .border(
            if target { 2.0 } else { 1.0 },
            Look::color(if target { pal.accent } else { pal.border }),
        )
        .on_hover_enter(move |_| {
            enter.hover(&key);
        })
        .on_click(move |_| {
            // A control or card inside has handled this click already and said so.
            if !click.swallowed() {
                click.drop_on(&ck);
            }
        })
        .child(head)
        .child(body)
}
