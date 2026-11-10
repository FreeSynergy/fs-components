//! The detail panel: every field of the type with its values (the commands are placed by `view/screens.rs`).
//!
//! Pattern: View function — session state in, elements out; the value components come from `view::value`.

use blinc_layout::div::{Div, div};
use fs_components_fields::{Look, Say};

use super::chrome::panel;
use super::value::show;
use crate::detail::Entry;

/// Label column and values, one row per field (the empty ones too: the Admin frontend shows all fields).
pub fn fields_panel(look: &Look, say: &dyn Say, entries: &[Entry]) -> Div {
    let (s, pal) = (look.style, look.palette());
    let mut panel = panel(look);
    for e in entries {
        let mut values = div()
            .flex_col()
            .gap_px(s.spacing.xs)
            .flex_grow_value(1.0)
            .min_w(0.0);
        if e.values.is_empty() {
            values = values.child(look.text(say.say("value.none"), s.text.body, pal.text_weak));
        }
        for v in &e.values {
            values = values.child(show(look, say, &e.field, v));
        }
        panel = panel.child(
            div()
                .class("fs-admin-field")
                .flex_row()
                .gap_px(s.spacing.m)
                .w_full()
                .child(div().w(s.text.body * 12.0).flex_shrink_0().child(look.text(
                    say.say(&e.field.label),
                    s.text.body,
                    pal.text_weak,
                )))
                .child(values),
        );
    }
    panel
}
