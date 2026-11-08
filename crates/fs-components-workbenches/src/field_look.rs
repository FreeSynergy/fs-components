//! The colours of a text field: text and ground from the theme of this build.
//!
//! Pattern: Decorator — wraps what `blinc_cn` 0.6.0 builds. Its input sets no text colour of its
//! own, so the text takes a default the layer made once; the hand test showed white text while
//! typing. Here text (`TextPrimary`), placeholder and ground (`InputBg`) are set at every build, from
//! the one theme, so they come as a pair.

use blinc_cn::components::input::InputBuilder;
use blinc_theme::{ColorToken, ThemeState};

/// `field` with text and ground that belong together.
pub(crate) fn paired(field: InputBuilder) -> InputBuilder {
    let theme = ThemeState::get();
    let ground = theme.color(ColorToken::InputBg);
    field
        .text_color(theme.color(ColorToken::TextPrimary))
        .placeholder_color(theme.color(ColorToken::TextTertiary))
        .idle_bg_color(ground)
}
