//! A style, light or dark, and the Blinc things made from it.
//!
//! Pattern: Adapter — `fs-components-styles` data in, Blinc colours, text and icons out. This is
//! the only place the views learn what a style says.

use blinc_core::Color;
use blinc_layout::div::GenericFont;
use blinc_layout::prelude::text;
use blinc_layout::svg::svg;
use blinc_layout::text::Text;
use fs_components_styles::{Glyph, Palette, Rgba, Style, StyleId, style};

/// How a component looks: a style and whether it is dark.
#[derive(Debug, Clone, Copy)]
pub struct Look {
    /// The style.
    pub style: &'static Style,
    /// Dark appearance.
    pub dark: bool,
}

impl Look {
    /// The built-in style `id`, light or dark.
    pub fn new(id: StyleId, dark: bool) -> Self {
        Self {
            style: style(id),
            dark,
        }
    }

    /// The colours in use.
    pub fn palette(&self) -> &Palette {
        self.style.palette(self.dark)
    }

    /// A style colour as a Blinc colour.
    pub fn color(c: Rgba) -> Color {
        Color::rgba(
            f32::from(c.r) / 255.0,
            f32::from(c.g) / 255.0,
            f32::from(c.b) / 255.0,
            f32::from(c.a) / 255.0,
        )
    }

    /// A line of text in the style's first font, `size` pixels, in `color`, on one line.
    pub fn text(&self, content: impl Into<String>, size: f32, color: Rgba) -> Text {
        text(content)
            .size(size)
            .color(Self::color(color))
            .font_with_fallback(self.style.fonts[0], GenericFont::SansSerif)
            .line_height(self.style.text.body_line / self.style.text.body)
            .no_wrap()
    }

    /// The icon `glyph` as the style draws it, `size` pixels (None: the set's size), in `color`.
    pub fn icon(
        &self,
        glyph: Glyph,
        size: Option<f32>,
        color: Rgba,
    ) -> impl blinc_layout::prelude::ElementBuilder + use<> {
        let px = size.unwrap_or(self.style.icons.size);
        svg(self.style.icons.svg(glyph, color, Some(px)))
            .size(px, px)
            .color(Self::color(color))
            .stroke_width(self.style.icons.stroke)
    }
}
