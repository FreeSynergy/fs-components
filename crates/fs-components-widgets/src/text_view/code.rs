//! A code block.
//!
//! Pattern: Adapter — lines of monospace text in a box styled by the theme. Long lines wrap, so
//! nothing is cut off.

use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::look::Look;

/// The element of a code block.
pub(super) fn code(source: &str, look: &Look) -> Div {
    let mut block = div().class("fs-tv-block").class("fs-tv-code");
    for line in source.trim_end_matches('\n').split('\n') {
        let shown = if line.is_empty() { " " } else { line };
        block = block.child(text(shown).monospace().size(look.small).color(look.text));
    }
    block
}
