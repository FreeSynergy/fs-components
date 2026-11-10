//! Apple: measures after Apple's design language (macOS apps, regular size).
//!
//! Pattern: none (plain data). From `docs/en/components/styles/apple.md` (HIG): macOS text styles
//! (Body 13/16, Subheadline 11/14, Title 3 15/20, Title 1 22/26), control 28 (minimum 20), padding
//! about 12 around controls, system blue/red/green (light and dark), regular Liquid Glass for
//! sidebars and popovers. **The sheet marks these "not published" and they are written by memory,
//! to be measured from Apple's apps:** row heights, radii, blur and tint of the glass, focus ring,
//! the neutral palette. Font: SF is not free; Inter (OFL).

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
        provenance: Provenance::Partial,
        fonts: &["Inter", "Noto Sans", "DejaVu Sans"],
        spacing: Spacing {
            xs: 4.0,
            s: 8.0,
            m: 12.0,
            l: 20.0,
            xl: 24.0,
        },
        heights: Heights {
            control: 28.0,
            control_dense: 20.0,
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
            accent: c(0x0088FF),
            on_accent: c(0xFFFFFF),
            focus: c(0x0088FF),
            hover: c(0xEDEDED),
            selected: c(0xD6E6FF),
            error: c(0xD70015),
            success: c(0x34C759),
        },
        dark: Palette {
            ground: c(0x1E1E1E),
            panel: c(0x2A2A2C),
            field: c(0x2D2D2F),
            text: c(0xF5F5F7),
            text_weak: c(0x98989D),
            border: c(0x48484A),
            accent: c(0x0091FF),
            on_accent: c(0xFFFFFF),
            focus: c(0x0091FF),
            hover: c(0x3A3A3C),
            selected: c(0x0A3A78),
            error: c(0xFF6961),
            success: c(0x30D158),
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
        effects_dark: Effects {
            chrome: Surface::Glass {
                blur: 20.0,
                tint: c(0x2A2A2C).alpha(0xB3),
                saturation: 1.8,
            },
            overlay: Surface::Glass {
                blur: 30.0,
                tint: c(0x2D2D2F).alpha(0xC7),
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
