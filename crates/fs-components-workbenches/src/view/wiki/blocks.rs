//! The block editor of the Notion-like layout: the page text as blocks, each one a field of its own.
//!
//! Pattern: Memento-ish chunks — the draft the reader keeps is the whole Markdown text; here it is cut at blank lines
//! (not inside a code fence) into blocks, every block gets a text area, and a change in any area puts the blocks back
//! together as the draft. So the reader's Save, conflict and backup logic stays the one there is. The identity of a
//! block (UUIDv7) lives in the text model of the wiki and is kept by the diff the wiki applies on Save; this view does
//! not hold it.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::widgets::text_area::SharedTextAreaState;
use blinc_layout::widgets::text_area::focus_text_area;
use fs_components_widgets::{PageTitle, TitleLabels};

use super::block_view::{Blocks, Report};
use crate::page_info::WikiLabels;

/// Cut `text` into blocks at blank lines; a fenced code block is one block whatever it contains.
pub(crate) fn split_blocks(text: &str) -> Vec<String> {
    fs_components_editor::Doc::parse(text).blocks
}

/// The blocks put together again: one blank line between, a newline at the end.
pub(crate) fn join_blocks(blocks: &[String]) -> String {
    let mut text = blocks.join("\n\n");
    text.push('\n');
    text
}

/// Does `block` say `title` as a level-1 heading (`# `, same comparison as `leads_with_title`)?
fn says_title(block: &str, title: &str) -> bool {
    block
        .strip_prefix("# ")
        .is_some_and(|rest| rest.trim() == title.trim())
}

/// What Enter in the title does: the keyboard goes to the first block (read when Enter is pressed).
fn jump_to(blocks: Arc<Blocks>) -> impl Fn() + Send + Sync + 'static {
    move || focus_text_area(&blocks.area(0))
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
pub struct BlockEditor {
    /// Which edit of the reader this belongs to.
    pub edit: u64,
    title: PageTitle,
    head: Head,
    blocks: Arc<Blocks>,
    /// The document as it was at the last `load` / `take_ops` of the editor contract.
    pub(super) loaded: fs_components_editor::Doc,
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
        Self {
            edit,
            title: PageTitle::new(&TitleLabels::default(), title),
            head: Head {
                in_text,
                first_title: title.to_owned(),
                lead,
            },
            blocks: Blocks::new(&parts),
            loaded: fs_components_editor::Doc::parse(draft),
        }
    }

    /// The whole text now, from the title field and the blocks.
    fn whole(&self) -> impl Fn() -> String + Send + Sync + 'static {
        let (head, title, blocks) = (
            self.head.clone(),
            self.title.clone(),
            Arc::clone(&self.blocks),
        );
        move || assemble(&head, &title.text(), &blocks.values())
    }

    /// Does the text lead with its title as a heading (then the title field draws it; no second big title)?
    #[cfg(test)]
    pub fn lead_heading(&self) -> bool {
        self.head.in_text
    }

    /// The whole text now (what the reports hand to the reader; here only for the tests).
    pub fn text(&self) -> String {
        (self.whole())()
    }

    /// The text of every block now (for hosts that drive the window from outside, and for the tests).
    pub fn block_texts(&self) -> Vec<String> {
        self.blocks.values()
    }

    /// Is the title a heading line of the text (then it is block 0 of the document and no block of the editor)?
    pub(super) fn in_text(&self) -> bool {
        self.head.in_text
    }

    /// The text of the title field.
    pub(super) fn title_text(&self) -> String {
        self.title.text()
    }

    /// Put `text` into the title field.
    pub(super) fn set_title(&self, text: &str) {
        self.title.set_text(text);
    }

    /// Type into block `i` of the editor.
    pub(super) fn blocks_type(&self, i: usize, at: usize, text: &str) {
        self.blocks.type_at(i, at, text);
    }

    /// Is the block menu open?
    pub fn menu_open(&self) -> bool {
        self.blocks.menu_open()
    }

    /// Press Enter at the end of the last block (for pictures).
    pub fn press_enter_at_end(&self) {
        let last = self.blocks.len() - 1;
        let end = self
            .blocks
            .area(last)
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .value()
            .len();
        self.blocks.enter_at(last, end);
    }

    /// A new empty block after the last one, with `/` typed into it: the block menu is open there (for pictures).
    pub fn open_menu_in_new_block(&self) {
        let last = self.blocks.len() - 1;
        let end = self
            .blocks
            .area(last)
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .value()
            .len();
        self.blocks.enter_at(last, end);
        self.blocks.type_at(last + 1, 0, "/");
    }

    /// Type into block `i` of the editor, as the keyboard would.
    pub fn type_in(&self, i: usize, at: usize, text: &str) {
        self.blocks.type_at(i, at, text);
    }

    /// Press Enter in block `i` of the editor.
    pub(super) fn blocks_enter(&self, i: usize, at: usize) {
        self.blocks.enter_at(i, at);
    }

    /// How many blocks there are.
    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.blocks.len()
    }

    /// The area of block `i`.
    pub fn area(&self, i: usize) -> SharedTextAreaState {
        self.blocks.area(i)
    }

    /// The block that has the keyboard, if any.
    pub fn focused(&self) -> Option<usize> {
        self.blocks.focused()
    }

    /// Give the keyboard to block `i`, the cursor `column` characters into its first line.
    pub fn put_cursor(&self, i: usize, column: usize) {
        let area = self.blocks.area(i);
        {
            let mut d = area
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let width = d.lines.first().map_or(0, |l| l.chars().count());
            d.cursor = blinc_layout::widgets::text_area::TextPosition::new(0, column.min(width));
        }
        focus_text_area(&area);
    }

    /// The big title field above the blocks (Enter in it moves on to the first block).
    pub fn title_view(&self, labels: &WikiLabels, report: &Report) -> Div {
        self.title.set_placeholder(&labels.title_hint);
        let report = Arc::clone(report);
        let (head, blocks) = (self.head.clone(), Arc::clone(&self.blocks));
        let field = self
            .title
            .clone()
            .on_change(move |text| report(&assemble(&head, text, &blocks.values())))
            .on_enter(jump_to(Arc::clone(&self.blocks)));
        div()
            .class("fs-wk-b-title-field")
            .w_full()
            .child(field.build())
    }

    /// The column of blocks. `report` gets the whole text after each change.
    pub fn view(&self, labels: &WikiLabels, report: &Report) -> Div {
        self.blocks.view(
            &labels.block_hint,
            &labels.slash,
            report,
            Arc::new(self.whole()),
        )
    }
}

#[cfg(test)]
#[path = "blocks_tests.rs"]
mod tests;
