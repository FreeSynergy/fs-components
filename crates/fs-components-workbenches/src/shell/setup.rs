//! Theme setup shared by window and picture.
//!
//! Pattern: Facade over `fs-components-render` — the one implementation of the theme bundle and
//! the fonts. A program that embeds the reader in its own window adds [`styles`] to its theme
//! bundle (`fs_components_render::bundle(&styles())`) and calls [`register_fonts`] once.

use blinc_theme::ThemeBundle;

pub use fs_components_render::register_fonts;

/// The theme bundle: the component library's sheet and ours, on the platform's theme.
pub(super) fn bundle() -> ThemeBundle {
    fs_components_render::bundle(&styles())
}

/// The style sheet of the reader and of the widgets it uses; add it to the theme of the window
/// that holds a [`ReaderWidget`](crate::ReaderWidget). Without it the layout collapses.
pub fn styles() -> String {
    crate::view::styles()
}
