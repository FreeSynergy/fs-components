//! Theme setup shared by window and picture.
//!
//! Pattern: Factory — one function builds the theme bundle (cn defaults plus our sheets).

use blinc_theme::ThemeBundle;

/// The theme bundle: the component library's sheet and ours, on the platform's theme.
pub(super) fn bundle() -> ThemeBundle {
    blinc_cn::cn_bundle().with_css(crate::view::styles())
}

/// Fonts for symbols the system may lack: Blinc's bundled emoji subset (documents use emoji in
/// headings and lists).
pub(super) fn register_fonts() {
    blinc_noto_emoji::register();
}
