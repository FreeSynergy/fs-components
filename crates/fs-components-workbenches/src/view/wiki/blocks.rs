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
use blinc_layout::widgets::text_area::SharedTextAreaState;
use blinc_layout::widgets::text_area::focus_text_area;
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

/// What Enter in the title does: the keyboard goes to the first block (it is built when Enter is pressed).
fn jump_to(first: Option<SharedTextAreaState>) -> impl Fn() + Send + Sync + 'static {
    move || {
        if let Some(first) = &first {
            focus_text_area(first);
        }
    }
}

/// The first line of a page: its title as a level-1 heading.
#[derive(Clone)]
struct Head {
    /// The text began with its title as a heading: that heading is the title field, not a block.
    in_text: bool,
    /// The title the page had when the edit began.
    first_title: String,
    /// The heading block as it stood in the text (kept while the title is not changed, so spaces survive).
    lead: String,
}

impl Head {
    /// The heading line for the title field `title`: the original while unchanged; `# title` when the person changed
    /// it (a line break in it becomes a space, so it stays one line); none for a blank title (the heading goes).
    fn line(&self, title: &str) -> Option<String> {
        let one_line = title.replace(['\n', '\r'], " ");
        let title = one_line.trim();
        if title.is_empty() {
            return None;
        }
        if self.in_text && title == self.first_title.trim() {
            return Some(self.lead.clone());
        }
        (self.in_text || title != self.first_title.trim()).then(|| format!("# {title}"))
    }
}

/// The text of a page put together: the heading line, then the blocks that are not blank. A page that is only its
/// title has one empty block (so there is a place to type), which is left out here.
fn assemble(head: &Head, title: &str, blocks: &[String]) -> String {
    let mut parts: Vec<String> = head.line(title).into_iter().collect();
    parts.extend(blocks.iter().filter(|b| !b.trim().is_empty()).cloned());
    join_blocks(&parts)
}

/// The editor of one edit: the title field and the blocks with their areas.
pub(crate) struct BlockEditor {
    /// Which edit of the reader this belongs to.
    pub edit: u64,
    title: PageTitle,
    head: Head,
    areas: Vec<(SharedTextAreaState, FieldData)>,
}

impl BlockEditor {
    /// The editor for `draft`, a page called `title`. A first block that is a level-1 heading saying the title is
    /// the title field and no block; another first heading stays a block. There is always at least one block.
    pub fn new(edit: u64, draft: &str, title: &str) -> Self {
        let mut parts = split_blocks(draft);
        let in_text = parts.first().is_some_and(|b| says_title(b, title));
        let lead = if in_text {
            parts.remove(0)
        } else {
            String::new()
        };
        if parts.is_empty() {
            parts.push(String::new());
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
            head: Head {
                in_text,
                first_title: title.to_owned(),
                lead,
            },
            areas,
        }
    }

    /// Does the text lead with its title as a heading (then the title field draws it; no second big title)?
    #[cfg(test)]
    pub fn lead_heading(&self) -> bool {
        self.head.in_text
    }

    /// The whole text now (what the reports hand to the reader; here only for the tests).
    #[cfg(test)]
    pub fn text(&self) -> String {
        let all: Vec<SharedTextAreaState> = self.areas.iter().map(|(a, _)| a.clone()).collect();
        assemble(&self.head, &self.title.text(), &values(&all))
    }

    /// The area of block `i` (tests look at its keyboard).
    #[cfg(test)]
    pub fn area(&self, i: usize) -> &SharedTextAreaState {
        &self.areas[i].0
    }

    /// The big title field above the blocks (Enter in it moves on to the first block).
    pub fn title_view(&self, labels: &WikiLabels, report: &Arc<dyn Fn(&str) + Send + Sync>) -> Div {
        self.title.set_placeholder(&labels.title_hint);
        let all: Arc<Vec<SharedTextAreaState>> =
            Arc::new(self.areas.iter().map(|(a, _)| a.clone()).collect());
        let next = all.first().cloned();
        let (report, head) = (Arc::clone(report), self.head.clone());
        let field = self
            .title
            .clone()
            .on_change(move |text| report(&assemble(&head, text, &values(&all))))
            .on_enter(jump_to(next));
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
            let (title, head) = (self.title.clone(), self.head.clone());
            let rows = rows_for(&state.lock().unwrap_or_else(PoisonError::into_inner).value());
            let area = textarea(state)
                .w_full()
                .rows(rows)
                .placeholder(&labels.block_hint)
                .on_change(move |_| {
                    report(&assemble(&head, &title.text(), &values(&all)));
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
        column
    }
}

#[cfg(test)]
#[path = "blocks_tests.rs"]
mod tests;
