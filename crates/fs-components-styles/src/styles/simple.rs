//! Simple: few large things, high contrast, for people with little experience.
//!
//! Pattern: none (plain data). Our own design, no outside sheet. Atkinson Hyperlegible (OFL) is made
//! for readability.

use crate::color::Rgba;
use crate::effects::{Effects, FocusRing, Shadow, Surface};
use crate::icons::{Cap, IconSet, Join};
use crate::style::{Provenance, Style, StyleId};
use crate::tokens::{Heights, Palette, Radii, Spacing, Type};

const fn c(rgb: u32) -> Rgba {
    Rgba::hex(rgb)
}

/// The Simple style.
pub fn build() -> Style {
    Style {
        id: StyleId::Simple,
        provenance: Provenance::Sheet,
        fonts: &["Atkinson Hyperlegible", "Noto Sans", "DejaVu Sans"],
        spacing: Spacing {
            xs: 8.0,
            s: 12.0,
            m: 20.0,
            l: 32.0,
            xl: 48.0,
        },
        heights: Heights {
            control: 56.0,
            control_dense: 48.0,
            row: 64.0,
            row_dense: 56.0,
            head: 48.0,
        },
        text: Type {
            caption: 16.0,
            body: 20.0,
            body_line: 28.0,
            title: 24.0,
            headline: 32.0,
            label_weight: 600,
        },
        radii: Radii {
            control: 14.0,
            card: 20.0,
            overlay: 24.0,
        },
        light: Palette {
            ground: c(0xFFFFFF),
            panel: c(0xF4F4F0),
            field: c(0xFFFFFF),
            text: c(0x111111),
            text_weak: c(0x4A4A4A),
            border: c(0x6B6B6B),
            accent: c(0x0B57D0),
            on_accent: c(0xFFFFFF),
            hover: c(0xE8EEF9),
            selected: c(0xD3E3FD),
            error: c(0xB3261E),
            success: c(0x146C2E),
        },
        dark: Palette {
            ground: c(0x000000),
            panel: c(0x1A1A1A),
            field: c(0x000000),
            text: c(0xFFFFFF),
            text_weak: c(0xC8C8C8),
            border: c(0x9A9A9A),
            accent: c(0xA8C7FA),
            on_accent: c(0x041E49),
            hover: c(0x2A2A2A),
            selected: c(0x0842A0),
            error: c(0xF2B8B5),
            success: c(0x6DD58C),
        },
        effects: Effects {
            chrome: Surface::Solid,
            overlay: Surface::Solid,
            card_shadow: None,
            overlay_shadow: Some(Shadow {
                dx: 0.0,
                dy: 4.0,
                blur: 16.0,
                spread: 0.0,
                color: c(0x000000).alpha(0x40),
            }),
            focus: FocusRing {
                width: 4.0,
                offset: 2.0,
            },
        },
        icons: IconSet {
            stroke: 2.5,
            cap: Cap::Round,
            join: Join::Round,
            size: 32.0,
        },
    }
}
