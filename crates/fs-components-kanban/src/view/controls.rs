//! A control of an operation: live when the host supports it, locked with a hint when not.
//!
//! Pattern: State (supported or locked) as one function — the same control is drawn both ways, so
//! the layout does not change when the host starts to support the operation (ADR 0066: it unlocks
//! by itself). A locked control has no handler and says `kanban.soon`.

use blinc_layout::div::{Div, div};
use fs_components_fields::Look;

use crate::handle::KanbanBoard;

/// A small labelled control. `in_column`: the click bubbles up to a column that must ignore it; `on` runs on a click when `supported`; otherwise the hint shows.
pub fn control(
    b: &KanbanBoard,
    id: &str,
    label_key: &str,
    supported: bool,
    in_column: bool,
    on: impl Fn() + Send + Sync + 'static,
) -> Div {
    let (look, s, pal) = (b.look, b.look.style, b.look.palette());
    let color = if supported { pal.text } else { pal.text_weak };
    let me = b.clone();
    let mut d = div()
        .id(id)
        .flex_col()
        .padding_x_px(s.spacing.s)
        .padding_y_px(s.spacing.xs)
        .rounded(s.radii.control)
        .border(1.0, Look::color(pal.border))
        .child(look.text(b.say.say(label_key), s.text.caption, color));
    if supported {
        d = d
            .bg(Look::color(pal.field))
            .attr("tabindex", "0")
            .cursor(blinc_layout::element::CursorStyle::Pointer)
            .on_click(move |_| {
                if in_column {
                    me.swallow();
                }
                on();
            });
    } else {
        d = d
            .child(look.text(b.say.say("kanban.soon"), s.text.caption, pal.text_weak))
            .attr("data-locked", "true")
            // A locked control does nothing, and the column below it must not take its click.
            .on_click(move |_| {
                if in_column {
                    me.swallow();
                }
            });
    }
    d
}
