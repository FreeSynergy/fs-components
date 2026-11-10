//! Small drawing parts: colours of tones, a box at a rectangle.
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
