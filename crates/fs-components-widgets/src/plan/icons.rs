//! The small drawings of the plan screens, our own, named as the freedesktop icon names they stand for.
//!
//! Pattern: Flyweight table — a name in, an outline drawing out (24 x 24, a line in the colour of the text, so one
//! drawing serves the dark and the light theme; the `-symbolic` kind of icon). A name not in the table draws nothing.

use blinc_layout::div::{Div, div};
use blinc_layout::svg::svg;

use crate::icon::{Tone, tone_colour};

/// The inner part of the drawing called `name`.
fn shapes(name: &str) -> Option<&'static str> {
    Some(match name {
        "emblem-default" => {
            "<rect x='4' y='4' width='16' height='16' rx='4'/><path d='M8 12l3 3 5-6'/>"
        }
        "mail-inbox" => "<path d='M3 13h5l1 3h6l1-3h5'/><path d='M5 5h14l2 8v6H3v-6z'/>",
        "starred" => {
            "<path d='M12 3l2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1L3.2 9.5l6.1-.9z'/>"
        }
        "x-office-calendar" => {
            "<rect x='3' y='5' width='18' height='16' rx='3'/><path d='M3 10h18M8 3v4M16 3v4'/>"
        }
        "view-list" => "<path d='M12 3l9 5-9 5-9-5z'/><path d='M3 13l9 5 9-5'/>",
        "folder" => {
            "<rect x='3' y='4' width='18' height='5' rx='1'/><path d='M5 9v10h14V9M10 13h4'/>"
        }
        "emblem-ok" => "<rect x='4' y='3' width='16' height='18' rx='3'/><path d='M8 12l3 3 5-6'/>",
        "view-grid" => {
            "<rect x='3' y='3' width='7' height='18' rx='2'/><rect x='14' y='3' width='7' height='11' rx='2'/>"
        }
        _ => return None,
    })
}

/// Whether the plan screens draw an icon of this name.
#[must_use]
pub fn has_icon(name: &str) -> bool {
    shapes(name).is_some()
}

/// The icon called `name`, `px` pixels square, in a muted colour; empty for a name not in the table.
pub fn plan_icon(name: &str, px: f32) -> Div {
    let Some(inner) = shapes(name) else {
        return div();
    };
    let source = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'>{inner}</svg>"
    );
    let source = source.replace('\'', "\"");
    div().flex_shrink_0().child(
        svg(source)
            .size(px, px)
            .color(tone_colour(Tone::Muted, None)),
    )
}
