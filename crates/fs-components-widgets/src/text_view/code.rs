//! A code block.
//!
//! Pattern: Adapter — lines of monospace text in a box styled by the theme. A line never wraps (a
//! diagram would break): every code block, in every profile, scrolls sideways inside its own width.

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::{Scroll, ScrollPhysics};

use super::fallback::needs_fallback;
use super::flow::flow;
use super::inline::Face;
use super::look::Look;
use crate::text::Inline;

/// The element of a code block.
pub(super) fn code(source: &str, look: &Look) -> Div {
    // The sideways box carries the ground; the lines inside only keep their padding.
    let mut block = div().class("fs-tv-codelines");
    for line in source.trim_end_matches('\n').split('\n') {
        let shown = if line.is_empty() { " " } else { line };
        if needs_fallback(line) {
            // Characters the monospace font lacks: the line is set run by run, each in a font
            // that has its characters (spaces and indentation are kept).
            let face = Face {
                size: look.small,
                bold: false,
                mono: true,
                cell: false,
            };
            block = block.child(flow(&Inline::plain(line), face, look, None));
        } else {
            block = block.child(
                text(shown)
                    .monospace()
                    .no_wrap()
                    .size(look.small)
                    .color(look.text),
            );
        }
    }
    sideways(block, source, look)
}

/// The block in a box that scrolls sideways, scrollbar always shown (a diagram is never broken at the edge).
fn sideways(block: Div, source: &str, look: &Look) -> Div {
    let lines = source.trim_end_matches('\n').split('\n').count();
    #[allow(clippy::cast_precision_loss)]
    let height = lines as f32 * look.small * 1.3 + 64.0;
    let physics = std::sync::Arc::new(std::sync::Mutex::new(ScrollPhysics::default()));
    div()
        .class("fs-tv-block")
        .class("fs-tv-code")
        .w_full()
        .h(height)
        .child(
            Scroll::with_physics(physics)
                .horizontal()
                .scrollbar_always()
                .scrollbar_thin()
                .w_full()
                .h_full()
                .content(block.flex_shrink_0()),
        )
}
