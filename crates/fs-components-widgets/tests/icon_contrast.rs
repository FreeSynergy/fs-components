//! The edge of a page icon (a sheet of paper) reaches the least contrast against the light rows it stands on.
//!
//! Tests: model test over the colours (WCAG 2.x contrast, `fs_components_styles::Rgba::contrast`), no drawing.

use fs_components_styles::Rgba;
use fs_components_widgets::icon::{Glyph, MIN_ICON_CONTRAST, kind_colour};
use fs_components_widgets::sheet_edge;

/// The grounds a sheet stands on: the white page, the sidebar of the Notion frontend and its selected row.
const GROUNDS: [u32; 3] = [0x00FF_FFFF, 0x00F7_F6F3, 0x00EB_E9E4];

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

#[test]
fn the_edge_of_every_sheet_reaches_the_least_contrast_on_every_ground() {
    for glyph in SHEETS {
        let edge = sheet_edge(kind_colour(glyph).unwrap());
        for ground in GROUNDS {
            let c = edge.contrast(Rgba::hex(ground));
            assert!(
                c >= MIN_ICON_CONTRAST,
                "{glyph:?} on {ground:06x}: contrast {c:.2}"
            );
        }
    }
}

#[test]
fn the_pale_edge_of_before_would_fail_this_measure() {
    // Counter-test: the edge colour the sheet had in 0.0.16 (0.70, 0.76, 0.80) on the sidebar.
    let old = Rgba::hex(0x00B2_C2CC);
    assert!(old.contrast(Rgba::hex(GROUNDS[1])) < MIN_ICON_CONTRAST);
}
