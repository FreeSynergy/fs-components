//! The part under the fields: the sentence of the problem, and the suggestions to click.
//!
//! Pattern: Adapter over Blinc's row, text and click handler; what to say comes from the type
//! (`BasicField::problem`), what to suggest from the program (`Suggest`). A problem is judged only
//! after the person changed something (an empty new form shows no error).

use std::sync::Arc;
use std::sync::atomic::Ordering;

use blinc_layout::div::{Div, div};

use super::Look;
use super::input::Inner;

/// How many suggestions at most.
const LIMIT: usize = 5;

pub(super) fn hint(on: &Arc<Inner>) -> Div {
    let s = on.look.style;
    let pal = on.look.palette();
    let field = on.kind.field();
    let parts = on.parts();
    let mut column = div().flex_col().gap_px(s.spacing.xs).w_full();
    let problem = field
        .problem(&parts)
        .filter(|_| on.touched.load(Ordering::Relaxed));
    if let Some(problem) = problem {
        column = column.child(
            on.look
                .text(on.say.say(problem.key()), s.text.caption, pal.error),
        );
    }
    let Some(source) = &on.suggest else {
        return column;
    };
    let segment = on.last.load(Ordering::Relaxed);
    let found = source.suggest(field, segment, &parts, LIMIT);
    if found.is_empty() {
        return column;
    }
    let mut chips = div().flex_row().flex_wrap().gap_px(s.spacing.xs);
    for (k, candidate) in found.into_iter().enumerate() {
        let target = Arc::clone(on);
        let value = candidate.clone();
        let mut chip = div().class("fs-field-suggestion");
        if let Some(name) = &on.name {
            chip = chip.id(format!("{name}-suggest-{k}"));
        }
        chips = chips.child(
            chip.padding_x_px(s.spacing.s)
                .h(s.heights.control_dense)
                .items_center()
                .rounded(s.radii.control)
                .bg(Look::color(pal.hover))
                .cursor_pointer()
                .on_click(move |_| {
                    target.refs[segment].set_value(value.clone());
                    target.refs[segment].focus();
                    target.live.refresh_now();
                })
                .child(on.look.text(candidate, s.text.body, pal.text)),
        );
    }
    column.child(chips)
}
