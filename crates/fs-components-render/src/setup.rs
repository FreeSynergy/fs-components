//! Theme and fonts, the same for a window and a picture.
//!
//! Pattern: Factory — one function builds the theme bundle (the component library's sheet plus
//! the caller's sheet, which uses theme variables only).

use blinc_theme::ThemeBundle;

/// The theme bundle: the component library's sheet and `css`, on the platform's theme.
pub fn bundle(css: &str) -> ThemeBundle {
    blinc_cn::cn_bundle().with_css(css.to_owned())
}

/// Fonts for symbols the system may lack: Blinc's bundled emoji subset.
pub fn register_fonts() {
    blinc_noto_emoji::register();
}
