//! An icon by its NAME: the chosen icon theme first, our built-in set when the theme does not have it.
//!
//! Pattern: Chain of responsibility — theme (freedesktop Icon Theme Specification, with inheritance), then the
//! built-in glyph of the same name (`Glyph::from_icon_name`), then the glyph of an unknown file. One place for every
//! program (ADR 0066); `icon` stays as it was and draws a glyph directly.

use std::sync::{Arc, PoisonError, RwLock};

use blinc_layout::div::{Div, div};
use blinc_layout::svg::svg;

use crate::icon::{Glyph, Tone, icon, tone_colour};
use crate::icon_theme::IconTheme;

static CHOSEN: RwLock<Option<Arc<IconTheme>>> = RwLock::new(None);

/// Choose the icon theme names are looked up in; `None` = the built-in set only (the default).
pub fn set_icon_theme(theme: Option<IconTheme>) {
    *CHOSEN.write().unwrap_or_else(PoisonError::into_inner) = theme.map(Arc::new);
}

fn chosen() -> Option<Arc<IconTheme>> {
    CHOSEN
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

/// The source of an SVG of a theme that is drawn as it is: Blinc paints a whole picture in the colour of the text
/// unless the source names `currentColor`, so an invisible dot that does is added to a coloured picture.
pub(crate) fn kept_colours(source: &str) -> String {
    if source.contains("currentColor") {
        return source.to_string();
    }
    match source.rfind("</svg>") {
        Some(end) => format!(
            "{}<path d=\"M0 0h.01\" fill=\"currentColor\" fill-opacity=\"0\"/>{}",
            &source[..end],
            &source[end..]
        ),
        None => source.to_string(),
    }
}

/// The SVG of `name` in the chosen theme at `px`, if the theme has it as an SVG (a `png` is not drawn here).
fn from_theme(name: &str, px: f32) -> Option<String> {
    let theme = chosen()?;
    // The size of the picture asked for, as the whole pixels the specification matches on.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let wanted = px.max(1.0).round() as u32;
    let path = theme.lookup(name, wanted)?;
    if path.extension()? != "svg" {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

/// The icon called `name` (`edit-copy`, `folder`, `text-x-generic` ...), `px` pixels square.
pub fn icon_named(name: &str, px: f32, tone: Tone) -> Div {
    if let Some(source) = from_theme(name, px) {
        let wide = svg(kept_colours(&source)).size(px, px);
        let wide = if source.contains("currentColor") {
            wide.color(tone_colour(tone, None))
        } else {
            wide
        };
        return div().flex_shrink_0().child(wide);
    }
    div().flex_shrink_0().child(icon(
        Glyph::from_icon_name(name).unwrap_or(Glyph::FileUnknown),
        px,
        tone,
    ))
}
