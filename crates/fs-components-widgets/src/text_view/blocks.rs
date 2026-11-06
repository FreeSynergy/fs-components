//! One element per block.
//!
//! Pattern: Strategy by block kind — a `match` on [`Block`] picks the drawing function; the
//! fallback ([`Block::Plain`]) draws text as it is, so nothing is lost.

use blinc_layout::div::{Div, div};

use super::LinkHandler;
use super::code::code;
use super::inline::{Face, inline};
use super::list::list;
use super::look::Look;
use super::table::table;
use crate::text::{Block, Inline};

/// A column holding the elements of `blocks`.
pub(super) fn blocks(
    blocks: &[Block],
    look: &Look,
    on_link: Option<&LinkHandler>,
    depth: usize,
) -> Div {
    let mut column = div().flex_col();
    for block in blocks {
        column = column.child(one(block, look, on_link, depth));
    }
    column
}

/// How deep quotes and lists nest before the rest is shown as plain text (a deep tree would
/// overflow the stack of the layout).
const MAX_DEPTH: usize = 24;

/// The element of one block.
fn one(block: &Block, look: &Look, on_link: Option<&LinkHandler>, depth: usize) -> Div {
    if depth >= MAX_DEPTH && matches!(block, Block::List { .. } | Block::Quote(_)) {
        return paragraph(&Inline::plain(block.plain_text()), look, None);
    }
    match block {
        Block::Heading { level, text } => {
            let idx = usize::from(level.clamp(&1, &6) - 1);
            let face = Face {
                size: look.heading[idx],
                bold: true,
                mono: false,
                cell: false,
            };
            div()
                .class("fs-tv-block")
                .class("fs-tv-heading")
                .child(inline(text, face, look, on_link))
        }
        Block::Paragraph(text) => paragraph(text, look, on_link),
        Block::List {
            ordered,
            start,
            items,
        } => list(*ordered, *start, items, look, on_link, depth + 1),
        Block::Quote(inner) => div()
            .class("fs-tv-block")
            .class("fs-tv-quote")
            .child(blocks(inner, look, on_link, depth + 1)),
        Block::Table { align, rows } => table(align, rows, look, on_link),
        Block::Rule => div().class("fs-tv-block").class("fs-tv-rule"),
        // Text that could not be mapped: line by line as it is, with its indentation and blank
        // lines (a paragraph would run the lines together).
        Block::Code { text, .. } | Block::Plain(text) => code(text, look),
    }
}

/// A paragraph of body text.
fn paragraph(text: &Inline, look: &Look, on_link: Option<&LinkHandler>) -> Div {
    let face = Face {
        size: look.base,
        bold: false,
        mono: false,
        cell: false,
    };
    div()
        .class("fs-tv-block")
        .child(inline(text, face, look, on_link))
}
