//! Theme and fonts, the same for a window and a picture.
//!
//! Pattern: Facade over Blinc's theme and font set-up — one function builds the theme bundle (the component library's sheet plus
//! the caller's sheet, which uses theme variables only).

use std::sync::Once;

use blinc_theme::ThemeBundle;

/// The theme bundle: the component library's sheet and `css`, on the platform's theme.
///
/// When the program chose a style with [`crate::use_style`], the bundle is that style's (window and picture alike).
pub fn bundle(css: &str) -> ThemeBundle {
    match crate::chosen_style() {
        Some(id) => crate::style_bundle(id, css),
        None => blinc_cn::cn_bundle().with_css(css.to_owned()),
    }
}

/// Fonts for symbols the system may lack: Blinc's bundled emoji subset. Once per process: each
/// registration would add the font again.
pub fn register_fonts() {
    static ONCE: Once = Once::new();
    ONCE.call_once(blinc_noto_emoji::register);
    register_style_fonts();
}

/// `JetBrains` Mono (SIL OFL 1.1, `assets/fonts/OFL-JetBrainsMono.txt`) for the style that wants a mono face for its labels
/// (Craft). Once per process, and only when that style is chosen.
#[cfg(feature = "craft-font")]
fn register_style_fonts() {
    static ONCE: Once = Once::new();
    if chosen_style_wants_mono() {
        ONCE.call_once(|| {
            blinc_app::register_font(
                include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf").to_vec(),
            );
            blinc_app::register_font(
                include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf").to_vec(),
            );
        });
    }
}

/// Without the feature `craft-font` the Craft style keeps the system's mono face; no font is registered.
#[cfg(not(feature = "craft-font"))]
fn register_style_fonts() {}

#[cfg(feature = "craft-font")]
fn chosen_style_wants_mono() -> bool {
    crate::chosen_style() == Some(fs_components_styles::StyleId::Craft)
}
