//! The icon of a kind of thing (a folder, a sheet of paper) built of boxes: filled, in colour, with a soft shadow.
//!
//! Pattern: Composite — Blinc paints a whole picture in the colour of the text unless its source names
//! `currentColor`, so a picture with colours of its own is not an SVG here: a folder is three boxes, a sheet is a box
//! with a fold and the sign of its kind (a thin mono line, drawn in the colour of the kind) on it. The sizes follow
//! the size asked for, so the same code gives the icon of a list row and the big one of a tile.

use blinc_core::{Brush, Color, Gradient};
use blinc_layout::div::{Div, div};
use blinc_layout::svg::svg;
use fs_components_styles::{Rgba, StyleId, style};

use crate::icon::{Glyph, kind_colour};

/// How much darker than its kind colour the edge of a sheet is (0.0 = the kind colour, 1.0 = black): the edge is what
/// shows a sheet on a light row, so it must reach the contrast of [`crate::icon::MIN_ICON_CONTRAST`] there.
const EDGE_DARKEN: f32 = 0.3;

fn colour(c: Rgba) -> Color {
    Color::rgba(
        f32::from(c.r) / 255.0,
        f32::from(c.g) / 255.0,
        f32::from(c.b) / 255.0,
        f32::from(c.a) / 255.0,
    )
}

/// A colour channel 0.0..=1.0 as a byte.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn byte(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn mix(c: Color, to: f32, by: f32) -> Color {
    Color::rgba(
        c.r + (to - c.r) * by,
        c.g + (to - c.g) * by,
        c.b + (to - c.b) * by,
        c.a,
    )
}

/// A ground that is a little lighter at the top: the hint of depth.
fn ground(base: Color) -> Brush {
    Brush::Gradient(Gradient::linear(
        blinc_core::Point::new(0.0, 0.0),
        blinc_core::Point::new(0.0, 1.0),
        mix(base, 1.0, 0.22),
        mix(base, 0.0, 0.03),
    ))
}

fn boxed(left: f32, top: f32, wide: f32, high: f32) -> Div {
    div().absolute().left(left).top(top).w(wide).h(high)
}

/// The folder: a back with a tab and a lighter front, filled in `kind`.
fn folder(px: f32, kind: Color) -> Div {
    let dark = mix(kind, 0.0, 0.18);
    let shade = Color::rgba(0.0, 0.0, 0.0, 0.22);
    div()
        .relative()
        .w(px)
        .h(px)
        .flex_shrink_0()
        .child(
            boxed(px * 0.06, px * 0.14, px * 0.4, px * 0.2)
                .rounded(px * 0.06)
                .bg(dark),
        )
        .child(
            boxed(px * 0.06, px * 0.22, px * 0.88, px * 0.62)
                .rounded(px * 0.08)
                .background(ground(dark)),
        )
        .child(
            boxed(px * 0.06, px * 0.36, px * 0.88, px * 0.5)
                .rounded(px * 0.08)
                .background(ground(kind))
                .shadow_params(0.0, px * 0.03, px * 0.06, shade),
        )
}

/// The edge of a sheet of `kind`: the kind colour, darkened (see `EDGE_DARKEN`).
pub fn sheet_edge(kind: Rgba) -> Rgba {
    let c = mix(colour(kind), 0.0, EDGE_DARKEN);
    Rgba {
        r: byte(c.r),
        g: byte(c.g),
        b: byte(c.b),
        a: kind.a,
    }
}

/// The sheet: paper with a folded corner, and the sign of the kind on it.
fn sheet(px: f32, glyph: Glyph, kind: Color) -> Div {
    let edge = mix(kind, 0.0, EDGE_DARKEN);
    let shade = Color::rgba(0.0, 0.0, 0.0, 0.24);
    let set = style(StyleId::Admin).icons;
    let sign = svg(set.svg_sign(
        glyph,
        Rgba {
            r: byte(kind.r),
            g: byte(kind.g),
            b: byte(kind.b),
            a: 255,
        },
        Some(px),
    ))
    .size(px, px)
    .color(kind);
    div()
        .relative()
        .w(px)
        .h(px)
        .flex_shrink_0()
        .child(
            boxed(px * 0.2, px * 0.08, px * 0.6, px * 0.84)
                .rounded(px * 0.06)
                .background(ground(Color::rgba(0.98, 0.99, 1.0, 1.0)))
                .border(1.0, edge)
                .shadow_params(0.0, px * 0.03, px * 0.07, shade),
        )
        .child(
            boxed(px * 0.62, px * 0.08, px * 0.18, px * 0.18)
                .rounded(px * 0.03)
                .bg(mix(kind, 1.0, 0.7))
                .border(1.0, edge),
        )
        .child(boxed(0.0, 0.0, px, px).child(sign))
}

/// The icon of a kind of thing, `px` pixels square; `None` if `glyph` is not a kind.
pub fn kind_icon(glyph: Glyph, px: f32) -> Option<Div> {
    let kind = colour(kind_colour(glyph)?);
    Some(if glyph == Glyph::Folder {
        folder(px, kind)
    } else {
        sheet(px, glyph, kind)
    })
}

/// The folders of a person that a file window pins at the side (the user folders of the desktop).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinnedKind {
    /// The desktop.
    Desktop,
    /// What was downloaded.
    Downloads,
    /// Documents.
    Documents,
    /// Pictures.
    Pictures,
    /// Music.
    Music,
    /// Videos.
    Videos,
}

impl PinnedKind {
    /// The colour of the folder and the sign laid on its front (none for the desktop).
    fn look(self) -> (u32, Option<Glyph>) {
        match self {
            PinnedKind::Desktop => (0x003B_82C4, None),
            PinnedKind::Downloads => (0x002F_9E6E, Some(Glyph::Download)),
            PinnedKind::Documents => (0x00C4_8A2A, Some(Glyph::FileDoc)),
            PinnedKind::Pictures => (0x0045_8F8A, Some(Glyph::FileImage)),
            PinnedKind::Music => (0x00A0_5BB5, Some(Glyph::FileAudio)),
            PinnedKind::Videos => (0x00C0_504D, Some(Glyph::FileVideo)),
        }
    }
}

/// A folder in the colour of its kind with a small white sign on its front: the icon of a pinned user folder.
pub fn pinned_icon(kind: PinnedKind, px: f32) -> Div {
    let (rgb, sign) = kind.look();
    let colour = colour(Rgba::hex(rgb));
    let mut icon = folder(px, colour);
    if let Some(glyph) = sign {
        let set = style(StyleId::Admin).icons;
        let svg = svg(set.svg_sign(glyph, Rgba::hex(0x00FF_FFFF), Some(px * 0.62)))
            .size(px * 0.62, px * 0.62)
            .color(Color::rgba(1.0, 1.0, 1.0, 1.0));
        icon = icon.child(boxed(px * 0.19, px * 0.3, px * 0.62, px * 0.62).child(svg));
    }
    icon
}
