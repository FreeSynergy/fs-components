//! The block editor of the Notion-like layout: the page text as blocks, each one a field of its own.
//!
//! Pattern: Memento-ish chunks — the draft the reader keeps is the whole Markdown text; here it is cut at blank lines
//! (not inside a code fence) into blocks, every block gets a text area, and a change in any area puts the blocks back
//! together as the draft. So the reader's Save, conflict and backup logic stays the one there is. The identity of a
//! block (UUIDv7) lives in the text model of the wiki and is kept by the diff the wiki applies on Save; this view does
//! not hold it.

use std::sync::{Arc, PoisonError};

use blinc_cn::components::textarea::textarea;
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::text_area::SharedTextAreaState;
use fs_components_primitives::FieldData;
use fs_components_widgets::icon::{Glyph, Tone, icon};

use crate::page_info::WikiLabels;

/// Cut `text` into blocks at blank lines; a fenced code block is one block whatever it contains.
pub(crate) fn split_blocks(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut fence: Option<&str> = None;
    for line in text.lines() {
        let marker = line.trim_start();
        let tick = ["```", "~~~"].into_iter().find(|f| marker.starts_with(f));
        match (fence, tick) {
            (None, Some(f)) => fence = Some(f),
            (Some(f), Some(t)) if f == t => fence = None,
            _ => {}
        }
        if fence.is_none() && line.trim().is_empty() && tick.is_none() {
            if !current.is_empty() {
                out.push(current.join("\n"));
                current.clear();
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        out.push(current.join("\n"));
    }
    out
}

/// The blocks put together again: one blank line between, a newline at the end.
pub(crate) fn join_blocks(blocks: &[String]) -> String {
    let mut text = blocks.join("\n\n");
    text.push('\n');
    text
}

/// How many text rows a block needs: its lines, a line that wraps (about 75 characters in the column) counting again.
fn rows_for(text: &str) -> usize {
    text.lines()
        .map(|l| l.chars().count().div_ceil(75).max(1))
        .sum::<usize>()
        .clamp(1, 30)
}

/// The editor of one edit: the blocks and their areas.
pub(crate) struct BlockEditor {
    /// Which edit of the reader this belongs to.
    pub edit: u64,
    areas: Vec<(SharedTextAreaState, FieldData)>,
}

impl BlockEditor {
    /// The editor for `draft`.
    pub fn new(edit: u64, draft: &str) -> Self {
        let parts = split_blocks(draft);
        let areas = parts
            .iter()
            .map(|part| {
                let keep = FieldData::new();
                let state = crate::view::new_area(part, &keep);
                (state, keep)
            })
            .collect();
        Self { edit, areas }
    }

    /// The column of blocks. `report` gets the whole text after each change.
    pub fn view(&self, labels: &WikiLabels, report: &Arc<dyn Fn(&str) + Send + Sync>) -> Div {
        let mut column = div().class("fs-wk-blocks").flex_col().w_full().gap(1.0);
        let all: Arc<Vec<SharedTextAreaState>> =
            Arc::new(self.areas.iter().map(|(a, _)| a.clone()).collect());
        for (state, _) in &self.areas {
            let (report, all) = (Arc::clone(report), Arc::clone(&all));
            let rows = rows_for(&state.lock().unwrap_or_else(PoisonError::into_inner).value());
            let area = textarea(state)
                .w_full()
                .rows(rows)
                .placeholder(&labels.block_hint)
                .on_change(move |_| {
                    let blocks: Vec<String> = all
                        .iter()
                        .map(|a| a.lock().unwrap_or_else(PoisonError::into_inner).value())
                        .collect();
                    report(&join_blocks(&blocks));
                });
            column = column.child(
                div()
                    .class("fs-wk-block")
                    .class("fs-wk-block-editing")
                    .flex_row()
                    .items_start()
                    .child(
                        div()
                            .class("fs-wk-grip")
                            .child(icon(Glyph::More, 16.0, Tone::Muted)),
                    )
                    .child(div().class("fs-body-field").flex_1().min_w(0.0).child(area)),
            );
        }
        if self.areas.is_empty() {
            column = column.child(text(&labels.block_hint).class("fs-wk-quiet").no_cursor());
        }
        column
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_are_cut_at_blank_lines() {
        let got = split_blocks("# T\n\nA\nB\n\n- x\n- y\n");
        assert_eq!(got, ["# T", "A\nB", "- x\n- y"]);
    }

    #[test]
    fn a_fence_is_one_block() {
        let got = split_blocks("a\n\n```rust\nfn x() {}\n\nfn y() {}\n```\n\nb");
        assert_eq!(got, ["a", "```rust\nfn x() {}\n\nfn y() {}\n```", "b"]);
    }

    #[test]
    fn join_gives_a_newline_at_the_end() {
        assert_eq!(join_blocks(&["a".into(), "b".into()]), "a\n\nb\n");
    }
}
