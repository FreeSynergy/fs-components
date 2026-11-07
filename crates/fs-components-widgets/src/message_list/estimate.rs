//! A first guess of how tall a message is, before it has been laid out.
//!
//! Pattern: none. Only a start: the measured height replaces it as soon as the message
//! has been laid out. Counts characters per line, so it needs no text measuring.

#![allow(clippy::cast_precision_loss)]

use super::model::ChatMessage;
use crate::text::{Block, Inline};

const LINE: f32 = 21.0;
const CHAR: f32 = 7.6;
const CHROME: f32 = 54.0;
const TOOL_CARD: f32 = 96.0;

fn inline_lines(t: &Inline, chars: f32) -> f32 {
    (t.text.chars().count() as f32 / chars).ceil().max(1.0)
}

fn block_lines(b: &Block, chars: f32) -> f32 {
    match b {
        Block::Heading { text, .. } => inline_lines(text, chars) + 0.6,
        Block::Paragraph(t) => inline_lines(t, chars) + 0.5,
        Block::List { items, .. } => items
            .iter()
            .map(|i| {
                i.blocks
                    .iter()
                    .map(|b| block_lines(b, chars - 3.0))
                    .sum::<f32>()
            })
            .sum(),
        Block::Quote(inner) => inner.iter().map(|b| block_lines(b, chars - 3.0)).sum(),
        Block::Code { text, .. } => text.lines().count().max(1) as f32 + 0.8,
        Block::Table { rows, .. } => rows.len() as f32 * 1.6,
        Block::Rule => 1.0,
        Block::Plain(s) => {
            (s.chars().count() as f32 / chars).ceil().max(1.0) + s.matches('\n').count() as f32
        }
    }
}

/// The height in pixels of `m` in a column `width` pixels wide.
pub fn estimate(m: &ChatMessage, width: f32) -> f32 {
    let chars = ((width - 56.0) / CHAR).max(8.0);
    let lines: f32 = m.blocks.iter().map(|b| block_lines(b, chars)).sum();
    let mut h = CHROME + lines.max(1.0) * LINE;
    h += m.tool_calls.len() as f32 * TOOL_CARD;
    if !m.reasoning.is_empty() {
        h += 34.0;
    }
    if !m.references.is_empty() {
        h += 32.0;
    }
    if m.delivery == Some(super::model::Delivery::Failed) {
        h += 56.0;
    }
    h
}
