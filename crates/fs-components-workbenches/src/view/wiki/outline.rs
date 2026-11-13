//! The outline of a page: its headings, and the history list.
//!
//! Pattern: pure functions — a document in, a list of (level, text) out; elements from that.

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::{Block, Document};

use crate::page_info::{PageInfo, RevisionRow, WikiLabels};

/// Does the document start with a level-1 heading that is its title? Then the layout draws that heading big above the text
/// and it is not drawn again as a block. A first heading that says something else stays a block.
pub(super) fn leads_with_title(document: &Document) -> bool {
    matches!(
        document.blocks.first(),
        Some(Block::Heading { level: 1, text }) if text.text.trim() == document.title.trim()
    )
}

/// The headings of `document` as (level, text), in order, without the title heading (see [`leads_with_title`]).
pub(super) fn headings(document: &Document) -> Vec<(u8, String)> {
    let skip = usize::from(leads_with_title(document));
    document
        .blocks
        .iter()
        .skip(skip)
        .filter_map(|block| match block {
            Block::Heading { level, text } => Some((*level, text.text.clone())),
            _ => None,
        })
        .collect()
}

/// What the outline panel lists: the title (a heading too) and then the headings.
pub(super) fn outline_entries(document: &Document) -> Vec<(u8, String)> {
    let mut out = Vec::new();
    if leads_with_title(document) {
        out.push((1, document.title.trim().to_owned()));
    }
    out.extend(headings(document));
    out
}

/// The blocks below the title: a leading title heading is dropped (the layout draws the title big above).
pub(super) fn body(document: &Document) -> Vec<Block> {
    let skip = usize::from(leads_with_title(document));
    document.blocks[skip..].to_vec()
}

/// The outline panel (Confluence layout): one row per heading, indented by level.
pub(super) fn outline_panel(document: &Document, labels: &WikiLabels) -> Div {
    let list = outline_entries(document);
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
