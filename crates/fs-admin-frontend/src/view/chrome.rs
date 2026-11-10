//! The frame of a page: title, actions, message bar, buttons.
//!
//! Pattern: View function — pure functions from model data and the measures of the [`Look`] to elements. The
//! buttons are `blinc_cn`'s (focus and Tab for free, ADR 0065 point 6); their height is cn's, not the style's
//! (finding for Thread C / Blinc, `DESIGN.md`).

use std::sync::Arc;

use blinc_cn::cn;
use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use fs_components_fields::{Look, Say};
use fs_components_styles::Glyph;
use fs_components_widgets::Element;

use crate::messages::{Message, MessageKind};

/// What a click does.
pub type Action = Arc<dyn Fn() + Send + Sync>;

/// How loud a button is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weight {
    /// The one thing to do on this page.
    Primary,
    /// The other things.
    Plain,
    /// Deletes; asks first.
    Danger,
}

/// A `blinc_cn` button with an element id (for tests and automation); `on` runs on a click.
pub fn cn_button(
    label: &str,
    variant: ButtonVariant,
    name: &str,
    on: impl Fn() + Send + Sync + 'static,
) -> Box<dyn Element> {
    Box::new(
        cn::button(label.to_owned())
            .variant(variant)
            .on_click(move |_| on())
            .build_component()
            .id(name),
    )
}

/// A locked button for a command the core cannot carry out yet: it is there, it cannot be pressed.
pub fn locked_button(label: &str, name: &str) -> Box<dyn Element> {
    Box::new(
        cn::button(label.to_owned())
            .variant(ButtonVariant::Secondary)
            .disabled(true)
            .build_component()
            .id(name),
    )
}

/// A button with a label that is already text.
pub fn button(label: &str, weight: Weight, name: &str, on: Action) -> Box<dyn Element> {
    let variant = match weight {
        Weight::Primary => ButtonVariant::Primary,
        Weight::Plain => ButtonVariant::Secondary,
        Weight::Danger => ButtonVariant::Destructive,
    };
    cn_button(label, variant, name, move || on())
}

/// The title, and below it the actions (wrapping).
pub fn header(look: &Look, title: &str, actions: Vec<Div>) -> Div {
    let (s, pal) = (look.style, look.palette());
    // Two rows, so the title can never lie over a button: the title on its own line (clipped, never wider than the
    // page), the actions below it in a row that wraps when they do not fit (the fs-tasks probe: a bar of many commands
    // ran out of the window and the title overlapped the first button).
    let mut bar = div()
        .id("page-actions")
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap_px(s.spacing.s)
        .w_full();
    for a in actions {
        bar = bar.child(a);
    }
    div()
        .class("fs-admin-header")
        .flex_col()
        .gap_px(s.spacing.s)
        .w_full()
        .min_w(0.0)
        .child(
            div()
                .id("page-title")
                .w_full()
                .min_w(0.0)
                .overflow_clip()
                .child(look.text(title, s.text.headline, pal.text)),
        )
        .child(bar)
}

/// How many rows the wrapping bar of actions needs in `width` (an estimate from the label lengths: the real widths
/// come from the buttons), so a page can leave room under it for the list.
pub fn bar_rows(look: &Look, labels: &[String], width: f32) -> usize {
    let s = look.style;
    let (mut rows, mut used) = (1_usize, 0.0_f32);
    for l in labels {
        #[allow(clippy::cast_precision_loss)]
        let w = l.chars().count() as f32 * s.text.body * 0.62 + 2.0 * s.spacing.m;
        if used > 0.0 && used + w > width {
            rows += 1;
            used = 0.0;
        }
        used += w + s.spacing.s;
    }
    rows
}

/// The messages of the screen, errors marked by colour and the icon (not by colour alone).
pub fn message_bar(
    look: &Look,
    say: &dyn Say,
    messages: &[Message],
    dismiss: &Arc<dyn Fn(u64) + Send + Sync>,
) -> Div {
    let (s, pal) = (look.style, look.palette());
    let mut column = div().flex_col().gap_px(s.spacing.xs).w_full();
    for m in messages {
        let (color, glyph) = match m.kind {
            MessageKind::Success => (pal.success, Glyph::Check),
            MessageKind::Error => (pal.error, Glyph::Warning),
        };
        let text = say.say(&format!("message.{}", m.code));
        let id = m.id;
        let off = Arc::clone(dismiss);
        let mut row = div()
            .class("fs-admin-message")
            .id(format!("message-{}", m.id))
            .flex_row()
            .items_center()
            .gap_px(s.spacing.s)
            .padding_x_px(s.spacing.s)
            .padding_y_px(s.spacing.xs)
            .rounded(s.radii.control)
            .border(1.0, Look::color(color))
            .bg(Look::color(pal.panel))
            .w_full()
            .child(look.icon(glyph, None, color))
            .child(div().flex_grow_value(1.0).min_w(0.0).child(look.text(
                text,
                s.text.body,
                pal.text,
            )));
        if m.kind == MessageKind::Error {
            row = row.child(cn_button(
                &say.say("action.close"),
                ButtonVariant::Ghost,
                &format!("dismiss-{id}"),
                move || off(id),
            ));
        }
        column = column.child(row);
    }
    column
}

/// A panel under the page header: the Fiori card.
pub fn panel(look: &Look) -> Div {
    let (s, pal) = (look.style, look.palette());
    div()
        .class("fs-admin-panel")
        .flex_col()
        .gap_px(s.spacing.s)
        .padding_x_px(s.spacing.m)
        .padding_y_px(s.spacing.m)
        .rounded(s.radii.card)
        .border(1.0, Look::color(pal.border))
        .bg(Look::color(pal.panel))
        .w_full()
}
