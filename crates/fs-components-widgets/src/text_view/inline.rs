//! Inline text: one element per run of text.
//!
//! Pattern: Strategy by content — text without marks is one Blinc rich-text element (exact
//! wrapping); text with marks goes through `flow.rs`, because Blinc measures a rich text in one
//! font weight and draws bold and italic spans in another face, so lines with marks run past the
//! edge or overlap (see DESIGN.md, Blinc notes).

use blinc_layout::div::{Div, div};
use blinc_layout::styled_text::{StyledLine, StyledText, TextSpan};
use blinc_layout::{RichText, rich_text_styled};

use super::LinkHandler;
use super::flow::flow;
use super::look::Look;
use crate::text::Inline;

/// A paragraph longer than this many bytes goes through `flow.rs` even without marks: Blinc's
/// measuring of one very long rich text is slow (30 KB took about 8 s), many small runs are not.
const LONG_PLAIN: usize = 1500;

/// How the text of one block is set.
#[derive(Clone, Copy)]
pub(super) struct Face {
    pub size: f32,
    pub bold: bool,
    /// Set in the monospace font (code).
    pub mono: bool,
}

/// The element for `inline` set in `face`.
pub(super) fn inline(
    inline: &Inline,
    face: Face,
    look: &Look,
    on_link: Option<&LinkHandler>,
) -> Div {
    if uses_flow(inline) {
        return flow(inline, face, look, on_link);
    }
    let text = if inline.text.is_empty() {
        " "
    } else {
        &inline.text
    };
    let span = TextSpan::colored(0, text.len(), look.text);
    let rich: RichText = rich_text_styled(StyledText::from_lines(vec![StyledLine::new(
        text.to_string(),
        vec![span],
    )]))
    .size(face.size)
    .default_color(look.text);
    let rich = if face.bold { rich.bold() } else { rich };
    div().w_full().child(rich)
}

/// Does this text go through `flow.rs` (many small runs) instead of one rich text? Marks, line
/// breaks, characters that need another font, and very long text do.
pub(super) fn uses_flow(inline: &Inline) -> bool {
    !inline.marks.is_empty()
        || inline.text.len() > LONG_PLAIN
        || inline.text.contains('\n')
        || super::fallback::needs_fallback(&inline.text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_plain_text_goes_through_the_flow_not_one_slow_rich_text() {
        // 30 KB in one rich text took seconds in Blinc's measuring; the flow does not.
        let long = Inline::plain("lorem ipsum dolor sit amet ".repeat(1200));
        assert!(long.text.len() > 30_000 && uses_flow(&long));
        assert!(!uses_flow(&Inline::plain("a short plain paragraph")));
        assert!(uses_flow(&Inline::plain("two\nlines")));
    }
}
