//! An emphasised run.
//!
//! Pattern: Adapter — Blinc's plain `text` does not draw italic, a rich text with an italic span
//! does, so an emphasised run is one small rich text.

use blinc_core::Color;
use blinc_layout::div::FontFamily;
use blinc_layout::styled_text::{StyledLine, StyledText, TextSpan};
use blinc_layout::{RichText, rich_text_styled};

use super::flow::Style;

/// An emphasised run: a rich text with an italic span (Blinc's plain text has no italic).
pub(super) fn emphasised(
    piece: &str,
    style: &Style,
    size: f32,
    color: Color,
    bold: bool,
) -> RichText {
    let mut span = TextSpan::colored(0, piece.len(), color).with_italic(true);
    span.bold = style.strong || bold;
    span.strikethrough = style.strike;
    span.underline = style.link.is_some();
    span.code = style.code;
    let rich = rich_text_styled(StyledText::from_lines(vec![StyledLine::new(
        piece.to_string(),
        vec![span],
    )]))
    .size(size)
    .default_color(color);
    // A run in a fallback font (Indic scripts, symbols) is set in that family.
    match &style.font {
        Some(family) => rich.font_family(FontFamily::named(family.to_string())),
        None => rich,
    }
}
