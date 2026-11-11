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
