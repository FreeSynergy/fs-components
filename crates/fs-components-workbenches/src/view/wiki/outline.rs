//! The outline of a page: its headings, and the history list.
//!
//! Pattern: pure functions — a document in, a list of (level, text) out; elements from that.

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::{Block, Document};

use crate::page_info::{PageInfo, RevisionRow, WikiLabels};

/// The headings of `document` as (level, text), in order. A first level-1 heading that equals the title is the title itself,
/// not an entry.
pub(super) fn headings(document: &Document) -> Vec<(u8, String)> {
    let mut out = Vec::new();
    for (i, block) in document.blocks.iter().enumerate() {
        if let Block::Heading { level, text } = block {
            if i == 0 && *level == 1 {
                continue;
            }
            out.push((*level, text.text.clone()));
        }
    }
    out
}

/// The blocks below the title: a leading level-1 heading is dropped (the layout draws the title big above).
pub(super) fn body(document: &Document) -> Vec<Block> {
    match document.blocks.first() {
        Some(Block::Heading { level: 1, .. }) => document.blocks[1..].to_vec(),
        _ => document.blocks.clone(),
    }
}

/// The outline panel (Confluence layout): one row per heading, indented by level.
pub(super) fn outline_panel(document: &Document, labels: &WikiLabels) -> Div {
    let list = headings(document);
    let mut column = div().class("fs-wk-panel").flex_col().w_full();
    column = column.child(text(&labels.outline).class("fs-wk-panel-title").no_cursor());
    if list.is_empty() {
        return column.child(text(&labels.no_headings).class("fs-wk-quiet").no_cursor());
    }
    for (level, label) in list.into_iter().take(80) {
        let indent = f32::from(level.saturating_sub(2).min(3)) * 12.0;
        column = column.child(
            div()
                .class("fs-wk-outline-row")
                .pl(indent)
                .child(text(label).no_cursor()),
        );
    }
    column
}

/// The ticks at the right edge (Notion layout): a short line per heading, longer for a higher level.
pub(super) fn ticks(document: &Document) -> Div {
    let mut column = div().class("fs-wk-ticks").flex_col().items_end();
    for (level, _) in headings(document).into_iter().take(40) {
        let width = match level {
            1 | 2 => 22.0,
            3 => 16.0,
            _ => 10.0,
        };
        column = column.child(div().class("fs-wk-tick").w(width));
    }
    column
}

/// The history panel: the revisions the source knows, newest first.
pub(super) fn history_panel(info: &PageInfo, labels: &WikiLabels) -> Div {
    let mut column = div().class("fs-wk-panel").flex_col().w_full();
    column = column.child(text(&labels.history).class("fs-wk-panel-title").no_cursor());
    if info.revisions.is_empty() {
        return column.child(text(&labels.no_history).class("fs-wk-quiet").no_cursor());
    }
    for (i, RevisionRow { label, note }) in info.revisions.iter().enumerate() {
        let mut row = div()
            .class("fs-wk-rev-row")
            .flex_col()
            .child(text(label).no_cursor());
        if i == 0 {
            row = row.class("fs-wk-rev-current");
        }
        if !note.is_empty() {
            row = row.child(text(note).class("fs-wk-quiet").no_cursor());
        }
        column = column.child(row);
    }
    column
}
