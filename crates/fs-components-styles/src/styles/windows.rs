//! Windows: measures after Microsoft's Fluent 2.
//!
//! Pattern: none (plain data). Provisional: written from the public Fluent 2 guidelines by memory
//! (14/20 text, 32 px control, 4 px control corners, 8 px overlay corners, 4 px grid, Mica) until
//! Thread M's `docs/en/components/styles/windows.md` exists. Font: Segoe UI is not free; Noto Sans
//! (OFL) has similar measures.

use crate::color::Rgba;
use crate::effects::{Effects, FocusRing, Shadow, Surface};
use crate::icons::{Cap, IconSet, Join};
use crate::style::{Provenance, Style, StyleId};
use crate::tokens::{Heights, Palette, Radii, Spacing, Type};

const fn c(rgb: u32) -> Rgba {
    Rgba::hex(rgb)
}

/// The Windows style.
pub fn build() -> Style {
    Style {
        id: StyleId::Windows,
        provenance: Provenance::Provisional,
        fonts: &["Noto Sans", "Open Sans", "DejaVu Sans"],
        spacing: Spacing {
            xs: 4.0,
            s: 8.0,
            m: 12.0,
            l: 16.0,
            xl: 24.0,
        },
        heights: Heights {
            control: 32.0,
            control_dense: 24.0,
            row: 40.0,
            row_dense: 32.0,
            head: 32.0,
        },
        text: Type {
            caption: 12.0,
            body: 14.0,
            body_line: 20.0,
            title: 20.0,
            headline: 28.0,
            label_weight: 400,
        },
        radii: Radii {
            control: 4.0,
            card: 8.0,
            overlay: 8.0,
        },
        light: Palette {
            ground: c(0xF3F3F3),
            panel: c(0xFFFFFF),
            field: c(0xFFFFFF),
            text: c(0x242424),
            text_weak: c(0x616161),
            border: c(0x8A8A8A),
            accent: c(0x0F6CBD),
            on_accent: c(0xFFFFFF),
            hover: c(0xF0F0F0),
            selected: c(0xE0EDF9),
            error: c(0xB10E1C),
            success: c(0x107C10),
        },
        dark: Palette {
            ground: c(0x202020),
            panel: c(0x2B2B2B),
            field: c(0x2B2B2B),
            text: c(0xFFFFFF),
            text_weak: c(0xC5C5C5),
            border: c(0x9A9A9A),
            accent: c(0x479EF5),
            on_accent: c(0x000000),
            hover: c(0x383838),
            selected: c(0x0F3A5E),
            error: c(0xEEACB2),
            success: c(0x9FD89F),
        },
        effects: Effects {
            chrome: Surface::Backdrop {
                tint: c(0xF3F3F3).alpha(0xE6),
            },
            overlay: Surface::Solid,
            card_shadow: Some(Shadow {
                dx: 0.0,
                dy: 2.0,
                blur: 4.0,
                spread: 0.0,
                color: c(0x000000).alpha(0x24),
            }),
            overlay_shadow: Some(Shadow {
                dx: 0.0,
                dy: 8.0,
                blur: 16.0,
                spread: 0.0,
                color: c(0x000000).alpha(0x24),
            }),
            focus: FocusRing {
                width: 2.0,
                offset: 1.0,
            },
        },
        icons: IconSet {
            stroke: 1.5,
            cap: Cap::Butt,
            join: Join::Miter,
            size: 16.0,
        },
    }
}
