//! What the theme says about a chart: colours, series palette, text sizes.
//!
//! Pattern: Value object snapshot — read once per draw from `ThemeState`; nothing here is a
//! literal colour. Series colours are chosen from the theme's accents by `color::pick`, which
//! keeps them apart for all kinds of colour vision and legible against the surface.

use blinc_core::Color;
use blinc_theme::{ColorToken, ThemeState};

use crate::color::{self, Rgb};
use crate::layout::Metrics;
use crate::scene::{Paint, TextSize};

/// The accents offered to the palette, theme primary first.
const ACCENTS: [ColorToken; 7] = [
    ColorToken::Primary,
    ColorToken::Warning,
    ColorToken::Info,
    ColorToken::Secondary,
    ColorToken::Accent,
    ColorToken::Success,
    ColorToken::Error,
];

/// Colours and sizes of the current theme.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Look {
    pub(super) text: Color,
    pub(super) muted: Color,
    pub(super) grid: Color,
    pub(super) axis: Color,
    pub(super) background: Color,
    pub(super) series: Vec<Color>,
    /// Sizes of chart furniture, from the theme's type scale.
    pub metrics: Metrics,
}

fn rgb(c: Color) -> Rgb {
    [c.r, c.g, c.b]
}

impl Look {
    /// Read the theme (initialised by the program before any widget is built).
    pub fn current() -> Self {
        let theme = ThemeState::get();
        let t = theme.typography();
        let background = theme.color(ColorToken::Background);
        let accents: Vec<Rgb> = ACCENTS.iter().map(|&a| rgb(theme.color(a))).collect();
        let series = color::pick(&accents, rgb(background))
            .into_iter()
            .map(|c| Color::rgb(c[0], c[1], c[2]))
            .collect();
        Self {
            text: theme.color(ColorToken::TextPrimary),
            muted: theme.color(ColorToken::TextSecondary),
            grid: theme.color(ColorToken::Border),
            axis: theme.color(ColorToken::TextTertiary),
            background,
            series,
            metrics: Metrics {
                title: t.text_sm,
                axis_title: t.text_xs,
                label: t.text_xs,
                ..Metrics::default()
            },
        }
    }

    /// The colour of a role as sRGB, 0..=1 (no Blinc type in the public API).
    pub fn rgb(&self, p: Paint) -> [f32; 3] {
        let c = self.colour(p);
        [c.r, c.g, c.b]
    }

    /// The colour of a role.
    pub(crate) fn colour(&self, p: Paint) -> Color {
        match p {
            Paint::Series(i) if !self.series.is_empty() => self.series[i % self.series.len()],
            Paint::Series(_) | Paint::Text => self.text,
            Paint::Muted => self.muted,
            Paint::Grid => self.grid,
            Paint::Axis => self.axis,
            Paint::Background => self.background,
        }
    }

    /// Pixel size of a text class.
    pub fn px(&self, s: TextSize) -> f32 {
        self.metrics.px(s)
    }
}
