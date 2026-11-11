//! `icon`: one glyph of our own icon set (`fs-components-styles`) as an element, in a tone of the theme.
//!
//! Pattern: Adapter — the shapes are data in `fs-components-styles` (drawn by us, ADR 0066, no other icon set);
//! this is the one place that turns a glyph into a Blinc element, so every widget and workbench draws icons alike.
//! A file kind has its own colour, so a folder or a picture is known at a glance; the rest follows the theme.

use blinc_layout::svg::svg;
use blinc_theme::{ColorToken, ThemeState};
use fs_components_styles::{Rgba, StyleId, style};

pub use fs_components_styles::Glyph;

use crate::Element;

/// How an icon is coloured.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// The colour of text.
    Normal,
    /// The colour of secondary text.
    Muted,
    /// The accent of the theme.
    Accent,
    /// The colour that belongs to the kind of file the glyph shows (a folder is amber, a picture green, ...).
    Kind,
}

fn rgba(c: blinc_core::Color) -> Rgba {
    // Truncation is what is meant: a colour channel is 0.0..=1.0 here.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Rgba {
        r: byte(c.r),
        g: byte(c.g),
        b: byte(c.b),
        a: byte(c.a),
    }
}

/// The colour of the kind of file a glyph shows; other glyphs have none and read as text.
pub fn kind_colour(glyph: Glyph) -> Option<Rgba> {
    Some(Rgba::hex(match glyph {
        Glyph::Folder => 0x00E0_A31F,
        Glyph::FileImage => 0x002F_9E6E,
        Glyph::FileText => 0x006B_7A8C,
        Glyph::FileArchive => 0x00A0_6B2C,
        Glyph::FileAudio => 0x00B0_5BB5,
        Glyph::FileVideo => 0x00C0_504D,
        Glyph::FilePdf => 0x00D0_4437,
        Glyph::FileSheet => 0x002E_8B57,
        Glyph::FileDoc => 0x003B_6FC4,
        Glyph::FileSlides => 0x00D9_7A2B,
        Glyph::FileCode => 0x004A_86C8,
        Glyph::FileUnknown => 0x008A_949E,
        _ => return None,
    }))
}

/// The icon `glyph`, `px` pixels square, in `tone`.
pub fn icon(glyph: Glyph, px: f32, tone: Tone) -> impl Element + use<> {
    let theme = ThemeState::get();
    let colour = match tone {
        Tone::Kind => {
            kind_colour(glyph).unwrap_or_else(|| rgba(theme.color(ColorToken::TextPrimary)))
        }
        Tone::Normal => rgba(theme.color(ColorToken::TextPrimary)),
        Tone::Muted => rgba(theme.color(ColorToken::TextSecondary)),
        Tone::Accent => rgba(theme.color(ColorToken::Primary)),
    };
    let set = style(StyleId::Admin).icons;
    svg(set.svg(glyph, colour, Some(px)))
        .size(px, px)
        .color(blinc_core::Color::rgba(
            f32::from(colour.r) / 255.0,
            f32::from(colour.g) / 255.0,
            f32::from(colour.b) / 255.0,
            f32::from(colour.a) / 255.0,
        ))
        .stroke_width(set.stroke)
}

/// The glyph of a media type: what a folder or a file of this kind looks like. `None` = a folder is asked with
/// `folder`; an unknown type is [`Glyph::FileUnknown`].
pub fn glyph_of(media_type: &str, folder: bool) -> Glyph {
    if folder {
        return Glyph::Folder;
    }
    let (major, sub) = media_type.split_once('/').unwrap_or((media_type, ""));
    match major {
        "image" => Glyph::FileImage,
        "audio" => Glyph::FileAudio,
        "video" => Glyph::FileVideo,
        "text" => match sub {
            "csv" | "tab-separated-values" => Glyph::FileSheet,
            "html" | "css" | "javascript" => Glyph::FileCode,
            s if s.starts_with("x-") => Glyph::FileCode,
            _ => Glyph::FileText,
        },
        "application" => match sub {
            "pdf" => Glyph::FilePdf,
            "zip" | "gzip" | "x-tar" | "x-7z-compressed" | "x-rar-compressed" => Glyph::FileArchive,
            "json" | "xml" | "yaml" | "toml" | "sql" | "javascript" => Glyph::FileCode,
            s if s.contains("spreadsheet") || s.contains("excel") => Glyph::FileSheet,
            s if s.contains("presentation") || s.contains("powerpoint") => Glyph::FileSlides,
            s if s.contains("wordprocessing")
                || s.contains("opendocument.text")
                || s.contains("msword") =>
            {
                Glyph::FileDoc
            }
            _ => Glyph::FileUnknown,
        },
        _ => Glyph::FileUnknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{Glyph, glyph_of, kind_colour};

    #[test]
    fn a_media_type_has_the_glyph_of_its_kind() {
        assert_eq!(glyph_of("image/jpeg", false), Glyph::FileImage);
        assert_eq!(glyph_of("application/pdf", false), Glyph::FilePdf);
        assert_eq!(glyph_of("application/zip", false), Glyph::FileArchive);
        assert_eq!(glyph_of("text/markdown", false), Glyph::FileText);
        assert_eq!(glyph_of("text/x-rust", false), Glyph::FileCode);
        assert_eq!(glyph_of("text/csv", false), Glyph::FileSheet);
        assert_eq!(
            glyph_of(
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                false
            ),
            Glyph::FileSheet
        );
        assert_eq!(
            glyph_of("application/octet-stream", false),
            Glyph::FileUnknown
        );
        assert_eq!(glyph_of("", true), Glyph::Folder);
    }

    #[test]
    fn file_kinds_have_a_colour_and_actions_follow_the_theme() {
        assert!(kind_colour(Glyph::Folder).is_some());
        assert!(kind_colour(Glyph::Trash).is_none());
    }
}
