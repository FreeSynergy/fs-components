//! Craft: a dark launcher look with sharp corners, 1 px lines, tiles that share their lines, and small capitals in a mono
//! face (after the public screenshots of the `ArtCraft` Launcher, MIT OR Apache-2.0; only measures were taken, nothing copied).
//!
//! Pattern: none (plain data). Every number is in `CRAFT-MEASUREMENT.md` of this crate (measured on the 1180 x 900 pictures).
//! Body text is a sans (Inter, Noto Sans); labels, tabs, badges and buttons use [`MONO`]; the style sheet that puts them
//! together is [`super::craft_sheet`]. The light palette is the light variant of the same pictures; the dark one is the look.

use crate::color::Rgba;
use crate::effects::{Effects, FocusRing, Surface};
use crate::icons::{Cap, IconSet, Join};
use crate::style::{Provenance, Style, StyleId};
use crate::tokens::{Heights, Palette, Radii, Spacing, Type};

const fn c(rgb: u32) -> Rgba {
    Rgba::hex(rgb)
}

/// The mono face of labels, in order of preference: `JetBrains` Mono (SIL OFL 1.1, shipped with `fs-components-render`),
/// then two mono faces most systems have.
pub const MONO: &[&str] = &["JetBrains Mono", "DejaVu Sans Mono", "Noto Sans Mono"];

/// The accent colours of the numbered tiles (the colour box and the second half of a name), in order: blue, red, purple,
/// amber, teal, pink, green. A package takes one by its name, so it keeps it.
pub const ACCENTS: [Rgba; 7] = [
    c(0x2F7BF5),
    c(0xE8573F),
    c(0x8B5CF6),
    c(0xF2A516),
    c(0x12A58A),
    c(0xE0368F),
    c(0x7BB51C),
];

/// The lighter shade of each accent used for text on the dark ground (the `Craft` of `PhotoCraft`).
pub const ACCENTS_TEXT: [Rgba; 7] = [
    c(0x7FB0FF),
    c(0xFF8A70),
    c(0xB59BFF),
    c(0xFFC14D),
    c(0x3FD6B8),
    c(0xFF7FBF),
    c(0xA5D94A),
];

/// The accent index of a name: the same name always has the same one.
pub fn accent_index(name: &str) -> usize {
    let hash = name.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    (hash % ACCENTS.len() as u64) as usize
}

/// Colours that are not in [`Palette`]: the lines, the deep ground of fields and badges, the status colours of the badges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extras {
    /// A 1 px line between and around tiles.
    pub line: Rgba,
    /// The stronger line between two columns of tiles.
    pub line_strong: Rgba,
    /// Ground of a field or a badge, deeper than the window.
    pub deep: Rgba,
    /// Third text level (the dim line under a subtitle, the footer).
    pub text_dim: Rgba,
    /// Badge text: update waiting.
    pub badge_update: Rgba,
    /// Badge text: offered.
    pub badge_offer: Rgba,
    /// Badge text: installed.
    pub badge_done: Rgba,
}

/// The extras for an appearance.
pub fn extras(dark: bool) -> Extras {
    if dark {
        Extras {
            line: c(0x333436),
            line_strong: c(0x4F5051),
            deep: c(0x060607),
            text_dim: c(0x8A8B8C),
            badge_update: c(0xFFC14D),
            badge_offer: c(0x74AAFF),
            badge_done: c(0x3FD6B8),
        }
    } else {
        Extras {
            line: c(0xCECDCB),
            line_strong: c(0xA5A5A3),
            deep: c(0xE9E8E4),
            text_dim: c(0x6F6E6F),
            badge_update: c(0x8A5800),
            badge_offer: c(0x1A5BC4),
            badge_done: c(0x0F7A66),
        }
    }
}

/// The Craft style.
pub fn build() -> Style {
    let effects = Effects {
        chrome: Surface::Solid,
        overlay: Surface::Solid,
        card_shadow: None,
        overlay_shadow: None,
        focus: FocusRing {
            width: 1.0,
            offset: 2.0,
        },
    };
    Style {
        id: StyleId::Craft,
        provenance: Provenance::Sheet,
        fonts: &["Inter", "Noto Sans", "DejaVu Sans"],
        spacing: Spacing {
            xs: 4.0,
            s: 8.0,
            m: 12.0,
            l: 20.0,
            xl: 28.0,
        },
        heights: Heights {
            control: 34.0,
            control_dense: 28.0,
            row: 68.0,
            row_dense: 48.0,
            head: 36.0,
        },
        text: Type {
            caption: 11.0,
            body: 13.0,
            body_line: 18.0,
            title: 18.0,
            headline: 28.0,
            label_weight: 700,
        },
        radii: Radii {
            control: 0.0,
            card: 0.0,
            overlay: 0.0,
        },
        light: Palette {
            ground: c(0xF2F1EE),
            panel: c(0xF2F1EE),
            field: c(0xE9E8E4),
            text: c(0x101014),
            text_weak: c(0x5E5D5E),
            border: c(0xCECDCB),
            accent: c(0x101014),
            on_accent: c(0xF2F1EE),
            focus: c(0x1A5BC4),
            hover: c(0xE9E8E4),
            selected: c(0xDCDBD7),
            error: c(0xB3261E),
            success: c(0x0F7A66),
        },
        dark: Palette {
            ground: c(0x121316),
            panel: c(0x121316),
            field: c(0x060607),
            text: c(0xF2F1EE),
            text_weak: c(0x989998),
            border: c(0x333436),
            accent: c(0xF2F1EE),
            on_accent: c(0x121316),
            focus: c(0x74AAFF),
            hover: c(0x1B1C20),
            selected: c(0x25262A),
            error: c(0xFF8A70),
            success: c(0x3FD6B8),
        },
        effects,
        effects_dark: effects,
        icons: IconSet {
            stroke: 1.5,
            cap: Cap::Butt,
            join: Join::Miter,
            size: 18.0,
        },
    }
}
