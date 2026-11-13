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
use blinc_layout::widgets::text_area::focus_text_area_deferred;
use fs_components_primitives::FieldData;
use fs_components_widgets::icon::{Glyph, Tone, icon};
use fs_components_widgets::{PageTitle, TitleLabels};

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

/// Does `block` say `title` as a level-1 heading (`# `, same comparison as `leads_with_title`)?
fn says_title(block: &str, title: &str) -> bool {
    block
        .strip_prefix("# ")
        .is_some_and(|rest| rest.trim() == title.trim())
}

/// The text of every block area.
fn values(areas: &[SharedTextAreaState]) -> Vec<String> {
    areas
        .iter()
        .map(|a| a.lock().unwrap_or_else(PoisonError::into_inner).value())
        .collect()
}

/// The text of a page put together: the `title` as the first line (`# `) when `with_heading` and the title is not
/// blank, then the blocks.
pub(crate) fn assemble(title: &str, with_heading: bool, blocks: &[String]) -> String {
    let title = title.trim();
    let mut parts: Vec<String> = Vec::with_capacity(blocks.len() + 1);
    if with_heading && !title.is_empty() {
        parts.push(format!("# {title}"));
    }
    parts.extend(blocks.iter().cloned());
    join_blocks(&parts)
}

/// The editor of one edit: the title field and the blocks with their areas.
pub(crate) struct BlockEditor {
    /// Which edit of the reader this belongs to.
    pub edit: u64,
    title: PageTitle,
    /// The title the page had when the edit began.
    first_title: String,
    /// The text began with its title as a heading: that heading is the title field, not a block.
    in_text: bool,
    areas: Vec<(SharedTextAreaState, FieldData)>,
}

impl BlockEditor {
    /// The editor for `draft`, a page called `title`. A first block that is a level-1 heading saying the title is
    /// the title field and no block; another first heading stays a block.
    pub fn new(edit: u64, draft: &str, title: &str) -> Self {
        let mut parts = split_blocks(draft);
        let in_text = parts.first().is_some_and(|b| says_title(b, title));
        if in_text {
            parts.remove(0);
        }
        let areas = parts
            .iter()
            .map(|part| {
                let keep = FieldData::new();
                let state = crate::view::new_area(part, &keep);
                (state, keep)
            })
            .collect();
        Self {
            edit,
            title: PageTitle::new(&TitleLabels::default(), title),
            first_title: title.to_owned(),
            in_text,
            areas,
        }
    }

    /// Does the text lead with its title as a heading (then the title field draws it; no second big title)?
    #[cfg(test)]
    pub fn lead_heading(&self) -> bool {
        self.in_text
    }

    /// The whole text now (what the reports hand to the reader; here only for the round-trip tests): the title field and the blocks put together (see [`assemble`]).
    #[cfg(test)]
    pub fn text(&self) -> String {
        let title = self.title.text();
        let all: Vec<SharedTextAreaState> = self.areas.iter().map(|(a, _)| a.clone()).collect();
        assemble(&title, self.keeps_heading(&title), &values(&all))
    }

    /// The title goes into the text when it led it, or when the person changed it.
    #[cfg(test)]
    fn keeps_heading(&self, title: &str) -> bool {
        self.in_text || title.trim() != self.first_title.trim()
    }

    /// The big title field above the blocks (Enter in it moves on to the first block).
    pub fn title_view(&self, labels: &WikiLabels, report: &Arc<dyn Fn(&str) + Send + Sync>) -> Div {
        let (in_text, first) = (self.in_text, self.first_title.clone());
        self.title.set_placeholder(&labels.title_hint);
        let all: Arc<Vec<SharedTextAreaState>> =
            Arc::new(self.areas.iter().map(|(a, _)| a.clone()).collect());
        let next = all.first().cloned();
        let report = Arc::clone(report);
        let field = self
            .title
            .clone()
            .on_change(move |text| {
                let with = in_text || text.trim() != first.trim();
                report(&assemble(text, with, &values(&all)));
            })
            .on_enter(move || {
                if let Some(first) = &next {
                    focus_text_area_deferred(first);
                }
            });
        div()
            .class("fs-wk-b-title-field")
            .w_full()
            .child(field.build())
    }

    /// The column of blocks. `report` gets the whole text after each change.
    pub fn view(&self, labels: &WikiLabels, report: &Arc<dyn Fn(&str) + Send + Sync>) -> Div {
        let mut column = div().class("fs-wk-blocks").flex_col().w_full().gap(1.0);
        let all: Arc<Vec<SharedTextAreaState>> =
            Arc::new(self.areas.iter().map(|(a, _)| a.clone()).collect());
        for (state, _) in &self.areas {
            let (report, all) = (Arc::clone(report), Arc::clone(&all));
            let (title, with, first) = (self.title.clone(), self.in_text, self.first_title.clone());
            let rows = rows_for(&state.lock().unwrap_or_else(PoisonError::into_inner).value());
            let area = textarea(state)
                .w_full()
                .rows(rows)
                .placeholder(&labels.block_hint)
                .on_change(move |_| {
                    let name = title.text();
                    let with = with || name.trim() != first.trim();
                    report(&assemble(&name, with, &values(&all)));
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
    fn title_and_blocks_go_through_markdown_and_back_unchanged() {
        for text in [
            "# Handbook\n\nIntro\n\n- a\n- b\n",
            "# Handbook\n",
            "# Einleitung\n\nText\n",
            "Text only\n",
            "# Handbook\n\n```\na\n\nb\n```\n\nEnd\n",
        ] {
            let editor = BlockEditor::new(1, text, "Handbook");
            assert_eq!(editor.text(), text, "round trip of {text:?}");
        }
    }

    #[test]
    fn the_title_is_not_a_block() {
        let editor = BlockEditor::new(1, "# Handbook\n\nIntro\n", "Handbook");
        assert!(editor.lead_heading());
        assert_eq!(editor.areas.len(), 1);
        assert_eq!(editor.title.text(), "Handbook");
        // Counter-test: a first heading that differs from the title stays a block.
        let other = BlockEditor::new(1, "# Einleitung\n\nIntro\n", "Handbook");
        assert!(!other.lead_heading());
        assert_eq!(other.areas.len(), 2);
    }

    #[test]
    fn a_changed_title_is_written_as_the_heading() {
        let editor = BlockEditor::new(1, "# Handbook\n\nIntro\n", "Handbook");
        editor.title.set_text("Guide");
        assert_eq!(editor.text(), "# Guide\n\nIntro\n");
        // A title that was not in the text (a page with another first heading) is added only once it is changed.
        let other = BlockEditor::new(1, "Intro\n", "Handbook");
        assert_eq!(other.text(), "Intro\n");
        other.title.set_text("Guide");
        assert_eq!(other.text(), "# Guide\n\nIntro\n");
    }

    #[test]
    fn join_gives_a_newline_at_the_end() {
        assert_eq!(join_blocks(&["a".into(), "b".into()]), "a\n\nb\n");
    }
}
