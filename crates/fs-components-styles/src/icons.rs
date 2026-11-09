//! The icon set: our own glyphs on a 24 x 24 grid, drawn with the line parameters of a style.
//!
//! Pattern: Value object (`IconSet` holds how to draw) over a closed enum of glyphs. The shapes are
//! drawn by us for this project; none is taken from another icon set (ADR 0066, item 6). What a
//! style changes is the line: width, ends, corners and the default size. A style with entirely
//! different shapes would add a second path table; that is not built (Unsupported).

use crate::color::Rgba;

/// What an icon shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Glyph {
    /// An envelope: e-mail address.
    Mail,
    /// A handset: phone number.
    Phone,
    /// A head and shoulders: person, name.
    Person,
    /// A map pin: place, postal address.
    Place,
    /// A calendar page: date.
    Calendar,
    /// A magnifier.
    Search,
    /// A plus.
    Plus,
    /// A cross.
    Close,
    /// A tick.
    Check,
    /// A triangle with a mark.
    Warning,
    /// Two sheets.
    Copy,
}

impl Glyph {
    /// All glyphs, for tests and for icon sheets.
    pub const ALL: [Glyph; 11] = [
        Glyph::Mail,
        Glyph::Phone,
        Glyph::Person,
        Glyph::Place,
        Glyph::Calendar,
        Glyph::Search,
        Glyph::Plus,
        Glyph::Close,
        Glyph::Check,
        Glyph::Warning,
        Glyph::Copy,
    ];

    /// The outline as SVG path data on a 24 x 24 grid.
    pub const fn path(self) -> &'static str {
        match self {
            Glyph::Mail => "M4 6h16v12H4z M4 7l8 6 8-6",
            Glyph::Phone => {
                "M7 3h3l1.5 4-2 1.5a11 11 0 0 0 5 5L16.5 11.5 21 13v3a2 2 0 0 1-2 2A16 16 0 0 1 5 5a2 2 0 0 1 2-2z"
            }
            Glyph::Person => "M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8z M4 20c0-4 3.5-6 8-6s8 2 8 6",
            Glyph::Place => {
                "M12 21s7-6.2 7-11a7 7 0 1 0-14 0c0 4.8 7 11 7 11z M12 12a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z"
            }
            Glyph::Calendar => "M4 6h16v14H4z M4 10h16 M8 3v4 M16 3v4",
            Glyph::Search => "M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14z M16 16l4.5 4.5",
            Glyph::Plus => "M12 5v14 M5 12h14",
            Glyph::Close => "M6 6l12 12 M18 6L6 18",
            Glyph::Check => "M5 12.5l4.5 4.5L19 7.5",
            Glyph::Warning => "M12 4L21 20H3z M12 10v4.5 M12 17.2v.3",
            Glyph::Copy => "M8 8h11v12H8z M5 16V4h11",
        }
    }

    /// A stable name (for tests, for translation keys of tooltips).
    pub const fn name(self) -> &'static str {
        match self {
            Glyph::Mail => "mail",
            Glyph::Phone => "phone",
            Glyph::Person => "person",
            Glyph::Place => "place",
            Glyph::Calendar => "calendar",
            Glyph::Search => "search",
            Glyph::Plus => "plus",
            Glyph::Close => "close",
            Glyph::Check => "check",
            Glyph::Warning => "warning",
            Glyph::Copy => "copy",
        }
    }
}

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
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{size}\" height=\"{size}\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"{cap}\" stroke-linejoin=\"{join}\"><path d=\"{}\"/></svg>",
            color.css(),
            self.stroke,
            glyph.path()
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
