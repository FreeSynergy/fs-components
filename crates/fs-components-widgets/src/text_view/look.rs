//! What the theme says about text.
//!
//! Pattern: none — read once per build from the theme, passed down; nothing
//! here is a literal colour or size.

use blinc_core::Color;
use blinc_theme::{ColorToken, ThemeState};

/// Colours and sizes of the current theme, as the text view needs them.
#[derive(Clone, Copy)]
pub(super) struct Look {
    pub text: Color,
    pub link: Color,
    pub base: f32,
    pub small: f32,
    pub heading: [f32; 6],
    /// Line height as a MULTIPLIER of the font size (Blinc reads `line_height` so, never as pixels); 0 = the engine's own.
    pub line: f32,
}

impl Look {
    /// Read the theme (initialised by the shell before any widget is built).
    pub fn current() -> Self {
        let theme = ThemeState::get();
        let t = theme.typography();
        Self {
            text: theme.color(ColorToken::TextPrimary),
            link: theme.color(ColorToken::TextLink),
            base: t.text_base,
            small: t.text_sm,
            heading: [
                t.text_3xl,
                t.text_2xl,
                t.text_xl,
                t.text_lg,
                t.text_base,
                t.text_base,
            ],
            line: 0.0,
        }
    }

    /// The reading profile of a wiki page: 16 px body at 1.5, headings 36 / 28 / 22 / 18.
    pub fn reading() -> Self {
        Self {
            base: 16.0,
            small: 13.5,
            heading: [36.0, 28.0, 22.0, 18.0, 16.0, 16.0],
            line: 1.5,
            ..Self::current()
        }
    }
}
