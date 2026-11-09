//! Apple: measures after Apple's design language (macOS apps, regular size).
//!
//! Pattern: none (plain data). Provisional: written from the public Human Interface Guidelines by
//! memory (13 pt text, 22 pt regular control, 6 pt control corners, translucent sidebar) until
//! Thread M's `docs/en/components/styles/apple.md` exists. Font: SF Pro is not free; Inter (OFL)
//! has similar measures.

use crate::color::Rgba;
use crate::effects::{Effects, FocusRing, Shadow, Surface};
use crate::icons::{Cap, IconSet, Join};
use crate::style::{Provenance, Style, StyleId};
use crate::tokens::{Heights, Palette, Radii, Spacing, Type};

const fn c(rgb: u32) -> Rgba {
    Rgba::hex(rgb)
}

/// The Apple style.
pub fn build() -> Style {
    Style {
        id: StyleId::Apple,
        provenance: Provenance::Provisional,
        fonts: &["Inter", "Noto Sans", "DejaVu Sans"],
        spacing: Spacing {
            xs: 4.0,
            s: 8.0,
            m: 12.0,
            l: 20.0,
            xl: 24.0,
        },
        heights: Heights {
            control: 22.0,
            control_dense: 19.0,
            row: 28.0,
            row_dense: 24.0,
            head: 24.0,
        },
        text: Type {
            caption: 11.0,
            body: 13.0,
            body_line: 16.0,
            title: 15.0,
            headline: 22.0,
            label_weight: 400,
        },
        radii: Radii {
            control: 6.0,
            card: 10.0,
            overlay: 12.0,
        },
        light: Palette {
            ground: c(0xFFFFFF),
            panel: c(0xF5F5F5),
            field: c(0xFFFFFF),
            text: c(0x1D1D1F),
            text_weak: c(0x6E6E73),
            border: c(0xD1D1D6),
            accent: c(0x007AFF),
            on_accent: c(0xFFFFFF),
            hover: c(0xEDEDED),
            selected: c(0xD6E6FF),
            error: c(0xFF3B30),
            success: c(0x28A745),
        },
        dark: Palette {
            ground: c(0x1E1E1E),
            panel: c(0x2A2A2C),
            field: c(0x2D2D2F),
            text: c(0xF5F5F7),
            text_weak: c(0x98989D),
            border: c(0x48484A),
            accent: c(0x0A84FF),
            on_accent: c(0xFFFFFF),
            hover: c(0x3A3A3C),
            selected: c(0x0A3A78),
            error: c(0xFF453A),
            success: c(0x32D74B),
        },
        effects: Effects {
            chrome: Surface::Glass {
                blur: 20.0,
                tint: c(0xF5F5F5).alpha(0xB3),
                saturation: 1.8,
            },
            overlay: Surface::Glass {
                blur: 30.0,
                tint: c(0xFFFFFF).alpha(0xC7),
                saturation: 1.8,
            },
            card_shadow: None,
            overlay_shadow: Some(Shadow {
                dx: 0.0,
                dy: 10.0,
                blur: 30.0,
                spread: 0.0,
                color: c(0x000000).alpha(0x38),
            }),
            focus: FocusRing {
                width: 3.0,
                offset: 0.0,
            },
        },
        icons: IconSet {
            stroke: 1.5,
            cap: Cap::Round,
            join: Join::Round,
            size: 16.0,
        },
    }
}
