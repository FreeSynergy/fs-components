//! The icon set: our own glyphs on a 24 x 24 grid, drawn with the line parameters of a style.
//!
//! Pattern: Value object (`IconSet` holds how to draw) over a closed enum of glyphs. The shapes are
//! drawn by us for this project; none is taken from another icon set (ADR 0066, item 6). What a
//! style changes is the line: width, ends, corners and the default size. A style with entirely
//! different shapes would add a second path table; that is not built (Unsupported).

use crate::color::Rgba;
use crate::glyph::Glyph;

/// How the end of a line is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cap {
    /// Cut off flat at the end point.
    Butt,
    /// Half circle.
    Round,
    /// Flat, half a line width beyond.
    Square,
}

/// How two lines meet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Join {
    /// Sharp point.
    Miter,
    /// Rounded.
    Round,
}

/// How a style draws the glyphs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconSet {
    /// Line width on the 24 grid.
    pub stroke: f32,
    /// Line ends.
    pub cap: Cap,
    /// Line corners.
    pub join: Join,
    /// Size in pixels when the caller does not say.
    pub size: f32,
}

impl IconSet {
    /// A complete SVG document of `glyph` in `color`, `size` pixels wide (None = the set's size).
    pub fn svg(&self, glyph: Glyph, color: Rgba, size: Option<f32>) -> String {
        let size = size.unwrap_or(self.size);
        let cap = match self.cap {
            Cap::Butt => "butt",
            Cap::Round => "round",
            Cap::Square => "square",
        };
        let join = match self.join {
            Join::Miter => "miter",
            Join::Round => "round",
        };
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{size}\" height=\"{size}\" viewBox=\"0 0 24 24\"><path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"{cap}\" stroke-linejoin=\"{join}\"/></svg>",
            glyph.path(),
            color.css(),
            self.stroke
        )
    }
}

/// The sheet of a file glyph with the fold of its corner; the rest of the outline is the sign on it.
const SHEET_OUTLINE: &str = "M6 3h8l4 4v14H6z M14 3v4h4 ";

impl Glyph {
    /// Does this glyph show a kind of thing (a folder or a file kind), so that it is drawn in colour with a body?
    pub const fn is_kind(self) -> bool {
        matches!(
            self,
            Glyph::Folder
                | Glyph::FileImage
                | Glyph::FileText
                | Glyph::FileArchive
                | Glyph::FileAudio
                | Glyph::FileVideo
                | Glyph::FilePdf
                | Glyph::FileSheet
                | Glyph::FileDoc
                | Glyph::FileSlides
                | Glyph::FileCode
                | Glyph::FileUnknown
        )
    }

    /// The sign on the sheet of a file glyph (what the outline has after the sheet); empty for the others.
    pub fn sign(self) -> &'static str {
        self.path().strip_prefix(SHEET_OUTLINE).unwrap_or("")
    }
}

impl IconSet {
    /// A complete SVG document of only the sign of a file glyph (what is drawn on the sheet), in `colour`: the sheet
    /// itself is drawn by the caller, so that it can have its own ground, edge and shadow.
    pub fn svg_sign(&self, glyph: Glyph, colour: Rgba, size: Option<f32>) -> String {
        let size = size.unwrap_or(self.size);
        let cap = match self.cap {
            Cap::Butt => "butt",
            Cap::Round => "round",
            Cap::Square => "square",
        };
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{size}\" height=\"{size}\" viewBox=\"0 0 24 24\"><path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"{cap}\" stroke-linejoin=\"round\"/></svg>",
            glyph.sign(),
            colour.css(),
            self.stroke * 0.9
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_has_a_distinct_name_and_a_path() {
        let mut names: Vec<_> = Glyph::ALL.iter().map(|g| g.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), Glyph::ALL.len());
        assert!(Glyph::ALL.iter().all(|g| g.path().starts_with('M')));
    }

    #[test]
    fn a_file_glyph_has_a_sign_and_the_others_have_none() {
        let set = IconSet {
            stroke: 1.5,
            cap: Cap::Round,
            join: Join::Round,
            size: 20.0,
        };
        assert!(Glyph::FilePdf.sign().starts_with('M'));
        assert_eq!(Glyph::Trash.sign(), "");
        let sign = set.svg_sign(Glyph::FilePdf, Rgba::hex(0xD04437), None);
        assert!(sign.contains("#d04437") && sign.contains(Glyph::FilePdf.sign()));
        assert!(
            !sign.contains("M6 3h8l4 4"),
            "the sheet is not part of the sign"
        );
        assert_eq!(Glyph::ALL.iter().filter(|g| g.is_kind()).count(), 12);
    }

    #[test]
    fn svg_carries_the_line_parameters() {
        let set = IconSet {
            stroke: 1.5,
            cap: Cap::Round,
            join: Join::Miter,
            size: 20.0,
        };
        let svg = set.svg(Glyph::Mail, Rgba::hex(0x112233), None);
        assert!(svg.contains("stroke-width=\"1.5\""));
        assert!(svg.contains("stroke-linecap=\"round\""));
        assert!(svg.contains("stroke-linejoin=\"miter\""));
        assert!(svg.contains("width=\"20\""));
        assert!(svg.contains("#112233"));
    }
}
