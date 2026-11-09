//! Admin: SAP Fiori (Horizon) measures for dense business data.
//!
//! Pattern: none (plain data). Provisional: written from the public Fiori guidelines (cozy control
//! 44 px, compact 32 px, 14 px text, 8 px grid, 8 px control corners) until Thread M's
//! `docs/en/components/styles/fiori.md` exists. Font: Fiori's own "72" is not free; Source Sans 3
//! (OFL) has similar measures.

use crate::color::Rgba;
use crate::effects::{Effects, FocusRing, Shadow, Surface};
use crate::icons::{Cap, IconSet, Join};
use crate::style::{Provenance, Style, StyleId};
use crate::tokens::{Heights, Palette, Radii, Spacing, Type};

const fn c(rgb: u32) -> Rgba {
    Rgba::hex(rgb)
}

/// The Admin style.
pub fn build() -> Style {
    Style {
        id: StyleId::Admin,
        provenance: Provenance::Provisional,
        fonts: &["Source Sans 3", "Noto Sans", "DejaVu Sans"],
        spacing: Spacing {
            xs: 4.0,
            s: 8.0,
            m: 16.0,
            l: 24.0,
            xl: 32.0,
        },
        heights: Heights {
            control: 44.0,
            control_dense: 32.0,
            row: 44.0,
            row_dense: 32.0,
            head: 44.0,
        },
        text: Type {
            caption: 12.0,
            body: 14.0,
            body_line: 20.0,
            title: 16.0,
            headline: 24.0,
            label_weight: 400,
        },
        radii: Radii {
            control: 8.0,
            card: 16.0,
            overlay: 16.0,
        },
        light: Palette {
            ground: c(0xF5F6F7),
            panel: c(0xFFFFFF),
            field: c(0xFFFFFF),
            text: c(0x1D2D3E),
            text_weak: c(0x556B82),
            border: c(0x758CA4),
            accent: c(0x0070F2),
            on_accent: c(0xFFFFFF),
            hover: c(0xEAF1FB),
            selected: c(0xD5E8FF),
            error: c(0xD20A0A),
            success: c(0x256F3A),
        },
        dark: Palette {
            ground: c(0x12171C),
            panel: c(0x1D232A),
            field: c(0x1D232A),
            text: c(0xEAECEE),
            text_weak: c(0xA9B4BE),
            border: c(0x8396A8),
            accent: c(0x58B6FF),
            on_accent: c(0x0A1623),
            hover: c(0x2A323B),
            selected: c(0x1F3A5A),
            error: c(0xFF8E8E),
            success: c(0x7CD59A),
        },
        effects: Effects {
            chrome: Surface::Solid,
            overlay: Surface::Solid,
            card_shadow: Some(Shadow {
                dx: 0.0,
                dy: 0.0,
                blur: 2.0,
                spread: 0.0,
                color: c(0x0D1F33).alpha(0x40),
            }),
            overlay_shadow: Some(Shadow {
                dx: 0.0,
                dy: 8.0,
                blur: 32.0,
                spread: 0.0,
                color: c(0x0D1F33).alpha(0x40),
            }),
            focus: FocusRing {
                width: 2.0,
                offset: 2.0,
            },
        },
        icons: IconSet {
            stroke: 1.75,
            cap: Cap::Butt,
            join: Join::Miter,
            size: 20.0,
        },
    }
}
