//! Paints as colours: the theme's tones, read when the picture is drawn.
//!
//! Pattern: Adapter — `Tone` to the theme's colour token. Nothing here is a literal colour of the
//! widget's; the caller's own RGBA passes through untouched.

use blinc_core::Color;
use blinc_theme::{ColorToken, ThemeState};

use super::model::{Paint, Tone};

fn token(tone: Tone) -> ColorToken {
    match tone {
        Tone::Surface => ColorToken::Surface,
        Tone::SurfaceElevated => ColorToken::SurfaceElevated,
        Tone::Border => ColorToken::Border,
        Tone::Text => ColorToken::TextPrimary,
        Tone::TextMuted => ColorToken::TextSecondary,
        Tone::Primary => ColorToken::Primary,
        Tone::Secondary => ColorToken::Secondary,
        Tone::Accent => ColorToken::Accent,
        Tone::Info => ColorToken::Info,
        Tone::Success => ColorToken::Success,
        Tone::Warning => ColorToken::Warning,
        Tone::Error => ColorToken::Error,
        _ => ColorToken::Background,
    }
}

/// The colour of a paint under the current theme (initialised by the program before any widget
/// is built).
pub fn colour(paint: Paint) -> Color {
    match paint {
        Paint::Theme(tone, opacity) => {
            let c = ThemeState::get().color(token(tone));
            let opacity = if opacity.is_finite() {
                opacity.clamp(0.0, 1.0)
            } else {
                1.0
            };
            Color::rgba(c.r, c.g, c.b, c.a * opacity)
        }
        Paint::Rgba(r, g, b, a) => Color::rgba(
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
            f32::from(a) / 255.0,
        ),
    }
}
