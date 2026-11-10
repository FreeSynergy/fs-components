//! One card: title, subtitle, labels, due mark, assignee initials, and the move buttons.
//!
//! Pattern: View function — a card and the state of the board in, an element out. The marks that
//! carry meaning (due state, cursor, held) use a glyph or a border as well as a colour.

use blinc_layout::div::{Div, div};
use fs_components_fields::Look;
use fs_components_styles::Glyph;

use super::controls::control;
use crate::handle::KanbanBoard;
use crate::layout::{card_height, initials};
use crate::model::{Card, DueState, Operation, Tone};
use crate::state::BoardState;

fn tone(look: &Look, t: Tone) -> fs_components_styles::Rgba {
    let p = look.palette();
    match t {
        Tone::Accent => p.accent,
        Tone::Success => p.success,
        Tone::Error => p.error,
        Tone::Neutral => p.text_weak,
    }
}

/// Height of the move buttons row of the selected card.
pub fn buttons_height(b: &KanbanBoard) -> f32 {
    (b.look.style.text.caption * 3.2).ceil() + b.look.style.spacing.xs
}

/// The card `c`, `width` pixels wide.
pub fn card(b: &KanbanBoard, st: &BoardState, c: &Card, column_key: &str, width: f32) -> Div {
    let (look, s, pal) = (b.look, b.look.style, b.look.palette());
    let selected = st.selected.as_deref() == Some(c.key.as_str());
    let held = st.held.as_ref().is_some_and(|h| h.key == c.key);
    let cursor = st.cursor.as_deref() == Some(c.key.as_str());
    let high = card_height(c, s) + if selected { buttons_height(b) } else { 0.0 };
    let (me, key) = (b.clone(), c.key.clone());
    let (hover, hk) = (b.clone(), column_key.to_owned());
    // The click target is a transparent layer above the content, not its parent: the buttons of the
    // card come after it as its siblings, so a click on a button does not reach the card's handler
    // (Blinc bubbles clicks to ancestors and has no way to stop them).
    let layer = div()
        .id(format!("kb-card-{}", c.key))
        .absolute()
        .inset(0.0)
        .cursor(blinc_layout::element::CursorStyle::Pointer)
        .on_click(move |_| {
            if me.snapshot().held.is_some() {
                me.drop_held();
            } else {
                me.select(&key);
            }
        })
        .on_hover_enter(move |_| {
            hover.hover(&hk);
        });
    let mut d = div()
        .flex_col()
        .gap_px(s.spacing.xs)
        .w_full()
        .padding_x_px(s.spacing.s)
        .padding_y_px(s.spacing.s)
        .child(look.text(&c.title, s.text.body, pal.text));
    if let Some(sub) = &c.subtitle {
        d = d.child(look.text(sub, s.text.caption, pal.text_weak));
    }
    if !c.labels.is_empty() {
        let mut row = div().flex_row().gap_px(s.spacing.xs);
        for l in &c.labels {
            let col = tone(&look, l.tone);
            row = row.child(
                div()
                    .padding_x_px(s.spacing.xs)
                    .rounded(s.radii.control)
                    .bg(Look::color(col.alpha(0x40)))
                    .child(look.text(&l.text, s.text.caption, pal.text)),
            );
        }
        d = d.child(row);
    }
    if c.due.is_some() || c.assignee.is_some() {
        d = d.child(footer(&look, c));
    }
    let mut outer = div()
        .id(format!("kb-box-{}", c.key))
        .relative()
        .w(width)
        .h(high)
        .rounded(s.radii.card)
        .bg(Look::color(if selected || held {
            pal.selected
        } else {
            pal.panel
        }))
        .border(
            if cursor || held { 2.0 } else { 1.0 },
            Look::color(if cursor || held {
                pal.accent
            } else {
                pal.border
            }),
        )
        .child(d)
        .child(layer);
    if selected {
        outer = outer.child(
            buttons(b, &c.key)
                .absolute()
                .left(s.spacing.s)
                .top(card_height(c, s)),
        );
    }
    outer
}

fn footer(look: &Look, c: &Card) -> Div {
    let (s, pal) = (look.style, look.palette());
    let mut row = div()
        .flex_row()
        .items_center()
        .gap_px(s.spacing.xs)
        .w_full();
    let mut due = div()
        .id(format!("kb-due-{}", c.key))
        .flex_row()
        .items_center()
        .gap_px(s.spacing.xs)
        .flex_grow_value(1.0);
    if let Some(d) = &c.due {
        let (glyph, col) = match d.state {
            DueState::Overdue => (Glyph::Warning, pal.error),
            DueState::Done => (Glyph::Check, pal.success),
            DueState::Soon | DueState::Open => (
                Glyph::Calendar,
                if d.state == DueState::Soon {
                    pal.accent
                } else {
                    pal.text_weak
                },
            ),
        };
        due = due
            .child(look.icon(glyph, Some(s.text.caption), col))
            .child(look.text(&d.text, s.text.caption, col));
    }
    row = row.child(due);
    if let Some(a) = &c.assignee {
        let size = s.text.caption * 2.0;
        row = row.child(
            div()
                .id(format!("kb-who-{}", c.key))
                .w(size)
                .h(size)
                .flex_row()
                .items_center()
                .justify_center()
                .rounded(size / 2.0)
                .bg(Look::color(pal.accent))
                .child(look.text(initials(a), s.text.caption, pal.on_accent)),
        );
    }
    row
}

fn buttons(b: &KanbanBoard, key: &str) -> Div {
    let s = b.look.style;
    let can = b.source.supports(Operation::Move);
    let make = |id: &str, label: &str, dir: i32| {
        let (me, k) = (b.clone(), key.to_owned());
        control(b, &format!("kb-{id}-{key}"), label, can, true, move || {
            me.step_over(&k, dir);
        })
    };
    let (me, k) = (b.clone(), key.to_owned());
    let held = b.snapshot().held.is_some();
    let grab = control(
        b,
        &format!("kb-grab-{key}"),
        if held { "kanban.drop" } else { "kanban.grab" },
        can,
        true,
        move || {
            if me.snapshot().held.is_some() {
                me.drop_held();
            } else {
                me.grab(&k);
            }
        },
    );
    div()
        .flex_row()
        .gap_px(s.spacing.xs)
        .child(make("left", "kanban.move_left", -1))
        .child(grab)
        .child(make("right", "kanban.move_right", 1))
}
