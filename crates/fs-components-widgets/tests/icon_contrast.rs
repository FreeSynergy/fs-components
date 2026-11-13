//! The edge of a page icon (a sheet of paper) reaches the least contrast against the light grounds of the theme.
//!
//! Tests: model test over the colours (WCAG 2.x contrast, `fs_components_styles::Rgba::contrast`), no drawing. The
//! grounds are colours of the theme (background, surface, raised surface), not hex values written here; the
//! sidebar of the Notion frontend is measured in its picture (`fs-components-workbenches/tests/wiki_icons.rs`).

mod common;

use blinc_theme::{ColorToken, ThemeState};
use fs_components_styles::Rgba;
use fs_components_widgets::icon::{Glyph, MIN_ICON_CONTRAST, kind_colour};
use fs_components_widgets::{sheet_edge, sheet_edge_by};

const SHEETS: [Glyph; 11] = [
    Glyph::FileText,
    Glyph::FileImage,
    Glyph::FileArchive,
    Glyph::FileAudio,
    Glyph::FileVideo,
    Glyph::FilePdf,
    Glyph::FileSheet,
    Glyph::FileDoc,
    Glyph::FileSlides,
    Glyph::FileCode,
    Glyph::FileUnknown,
];

fn grounds() -> Vec<Rgba> {
    let theme = ThemeState::get();
    [
        ColorToken::Background,
        ColorToken::Surface,
        ColorToken::SurfaceElevated,
    ]
    .map(|t| {
        let c = theme.color(t);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        Rgba {
            r: byte(c.r),
            g: byte(c.g),
            b: byte(c.b),
            a: 255,
        }
    })
    .to_vec()
}

#[test]
fn the_edge_of_every_sheet_reaches_the_least_contrast_on_every_ground() {
    let _serial = common::serial();
    for glyph in SHEETS {
        let edge = sheet_edge(kind_colour(glyph).unwrap());
        for ground in grounds() {
            let c = edge.contrast(ground);
            assert!(c >= MIN_ICON_CONTRAST, "{glyph:?}: contrast {c:.2}");
        }
    }
}

#[test]
fn without_the_darkening_a_sheet_would_fail_this_measure() {
    // Counter-test through the same code path: an edge that is the kind colour itself (darkening 0) is too pale for
    // at least one kind on at least one ground, so the darkening is what makes the test pass.
    let _serial = common::serial();
    let pale = SHEETS.iter().any(|&glyph| {
        let edge = sheet_edge_by(kind_colour(glyph).unwrap(), 0.0);
        grounds()
            .iter()
            .any(|g| edge.contrast(*g) < MIN_ICON_CONTRAST)
    });
    assert!(pale, "the measure cannot fail");
}
