//! What the theme says about text.
//!
//! Pattern: Value object snapshot — read once per build from the theme, passed down; nothing
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
        }
    }
}
