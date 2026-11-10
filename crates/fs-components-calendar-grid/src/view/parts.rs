//! Small drawing parts shared by the month and the time views.
//!
//! Pattern: View function — pure functions from data and the [`Look`] to elements.

use blinc_layout::div::{Div, div};
use fs_components_fields::Look;
use fs_components_styles::Rgba;

use crate::layout::Rect;
use crate::model::Tone;

/// The style colour of a tone.
pub fn tone_color(look: &Look, tone: Tone) -> Rgba {
    let p = look.palette();
    match tone {
        Tone::Accent => p.accent,
        Tone::Success => p.success,
        Tone::Error => p.error,
        Tone::Neutral => p.text_weak,
    }
}

/// An empty box at `r` inside a relatively placed parent.
pub fn at(r: Rect) -> Div {
    div().absolute().left(r.x).top(r.y).w(r.w).h(r.h)
}

/// A bar or block for an item: tinted ground, text, a border when selected.
pub fn chip(look: &Look, r: Rect, tone: Tone, title: &str, selected: bool) -> Div {
    let (s, pal) = (look.style, look.palette());
    let c = tone_color(look, tone);
    let mut d = at(r)
        .overflow_clip()
        .rounded(s.radii.control.min(4.0))
        .bg(Look::color(c.alpha(0x40)))
        .border(
            if selected { 2.0 } else { 1.0 },
            Look::color(if selected { pal.accent } else { c }),
        )
        .padding_x_px(s.spacing.xs)
        .child(look.text(title, s.text.caption, pal.text));
    d = d.cursor(blinc_layout::element::CursorStyle::Pointer);
    d
}

/// The number of a day; today gets a filled accent circle (and the number stays readable on it).
pub fn day_number(look: &Look, n: u8, today: bool, weak: bool) -> Div {
    let (s, pal) = (look.style, look.palette());
    let size = s.text.caption * 1.8;
    let color = if today {
        pal.on_accent
    } else if weak {
        pal.text_weak
    } else {
        pal.text
    };
    let mut d = div()
        .w(size)
        .h(size)
        .flex_row()
        .items_center()
        .justify_center()
        .rounded(size / 2.0);
    if today {
        d = d.bg(Look::color(pal.accent));
    }
    d.child(look.text(n.to_string(), s.text.caption, color))
}
