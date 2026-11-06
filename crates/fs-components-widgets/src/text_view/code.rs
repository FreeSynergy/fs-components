//! A code block.
//!
//! Pattern: Adapter — lines of monospace text in a box styled by the theme. Long lines wrap, so
//! nothing is cut off.

use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::fallback::needs_fallback;
use super::flow::flow;
use super::inline::Face;
use super::look::Look;
use crate::text::Inline;

/// The element of a code block.
pub(super) fn code(source: &str, look: &Look) -> Div {
    let mut block = div().class("fs-tv-block").class("fs-tv-code");
    for line in source.trim_end_matches('\n').split('\n') {
        let shown = if line.is_empty() { " " } else { line };
        if needs_fallback(line) {
            // Characters the monospace font lacks: the line is set run by run, each in a font
            // that has its characters (spaces and indentation are kept).
            let face = Face {
                size: look.small,
                bold: false,
                mono: true,
            };
            block = block.child(flow(&Inline::plain(line), face, look, None));
        } else {
            block = block.child(text(shown).monospace().size(look.small).color(look.text));
        }
    }
    block
}
