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

/// A button with a label that is already text.
pub fn button(label: &str, weight: Weight, name: &str, on: Action) -> Box<dyn Element> {
    let variant = match weight {
        Weight::Primary => ButtonVariant::Primary,
        Weight::Plain => ButtonVariant::Secondary,
        Weight::Danger => ButtonVariant::Destructive,
    };
    cn_button(label, variant, name, move || on())
}

/// The title and the actions to the right of it.
pub fn header(look: &Look, title: &str, actions: Vec<Div>) -> Div {
    let (s, pal) = (look.style, look.palette());
    let mut row = div()
        .class("fs-admin-header")
        .flex_row()
        .items_center()
        .gap_px(s.spacing.m)
        .w_full()
        .child(div().flex_grow_value(1.0).min_w(0.0).child(look.text(
            title,
            s.text.headline,
            pal.text,
        )));
    for a in actions {
        row = row.child(a);
    }
    row
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
