//! Theme setup shared by window and picture.
//!
//! Pattern: Factory — one function builds the theme bundle (cn defaults plus our sheets). A
//! program that embeds the reader in its own window adds [`styles`] to its theme bundle
//! (`cn_bundle().with_css(styles())`) and calls [`register_fonts`] once.

use blinc_theme::ThemeBundle;

/// The theme bundle: the component library's sheet and ours, on the platform's theme.
pub(super) fn bundle() -> ThemeBundle {
    blinc_cn::cn_bundle().with_css(crate::view::styles())
}

/// The style sheet of the reader and of the widgets it uses; add it to the theme of the window
/// that holds a [`ReaderWidget`](crate::ReaderWidget). Without it the layout collapses.
pub fn styles() -> String {
    crate::view::styles()
}

/// Fonts for symbols the system may lack: Blinc's bundled emoji subset (documents use emoji in
/// headings and lists).
pub fn register_fonts() {
    blinc_noto_emoji::register();
}
