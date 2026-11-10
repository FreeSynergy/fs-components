//! Windows: measures after Microsoft's Fluent 2.
//!
//! Pattern: none (plain data). From `docs/en/components/styles/windows.md` (Windows 11 and Fluent 2):
//! type ramp (Caption 12/16, Body 14/20, Subtitle 20/28, Title 28/36), corners 4 (controls) and 8
//! (overlays), spacing on a 4 px base, Mica as the base layer, flat surfaces. **Not published, written
//! by memory:** control and row heights, all colour values (the pages give token names only), shadows,
//! focus ring width. Font: Segoe UI is not free; Selawik (OFL, metric-compatible), Inter, Noto Sans.

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
        provenance: Provenance::Partial,
        fonts: &["Selawik", "Inter", "Noto Sans", "DejaVu Sans"],
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
            focus: c(0x0F6CBD),
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
            focus: c(0x479EF5),
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
