//! The block editor of the Notion-like layout: the page text as blocks on one writing surface, no field around a block.
//!
//! Pattern: Memento-ish chunks — the draft the reader keeps is the whole Markdown text; here it is cut at blank lines
//! (not inside a code fence) into blocks, which the surface (`surface.rs`) shows as one continuous text, and a change on it
//! puts the blocks back together as the draft. So the reader's Save, conflict and backup logic stays the one there is. The identity of a
//! block (UUIDv7) lives in the text model of the wiki and is kept by the diff the wiki applies on Save; this view does
//! not hold it.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use fs_components_widgets::{PageTitle, TitleLabels};

use super::surface::{Report, Surface};
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
fn jump_to(surface: Arc<Surface>) -> impl Fn() + Send + Sync + 'static {
    move || surface.put_cursor(0, 0)
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

/// The editor of one edit: the title field and the writing surface.
pub struct BlockEditor {
    /// Which edit of the reader this belongs to.
    pub edit: u64,
    title: PageTitle,
    head: Head,
    surface: Arc<Surface>,
    /// The document as it was at the last `load` / `take_ops` of the editor contract.
    pub(super) loaded: fs_components_editor::Doc,
}

impl BlockEditor {
    /// The editor for `draft`, a page called `title`. A first block that is a level-1 heading saying the title is
    /// the title field and no block; another first heading stays a block. There is always a place to write.
    pub fn new(edit: u64, draft: &str, title: &str) -> Self {
        let mut parts = split_blocks(draft);
        let in_text = parts.first().is_some_and(|b| says_title(b, title));
        let lead = if in_text {
            parts.remove(0)
        } else {
            String::new()
        };
        Self {
            edit,
            title: PageTitle::new(&TitleLabels::default(), title),
            head: Head {
                in_text,
                first_title: title.to_owned(),
                lead,
            },
            surface: Surface::new(&parts.join("\n\n")),
            loaded: fs_components_editor::Doc::parse(draft),
        }
    }

    /// The whole text now, from the title field and the surface.
    fn whole(&self) -> impl Fn() -> String + Send + Sync + 'static {
        let (head, title, surface) = (
            self.head.clone(),
            self.title.clone(),
            Arc::clone(&self.surface),
        );
        move || assemble(&head, &title.text(), &split_blocks(&surface.text()))
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

    /// The text of every block of the surface now, the empty one at the end if the person is about to write there (for hosts
    /// that drive the window from outside, and for the tests).
    pub fn block_texts(&self) -> Vec<String> {
        self.surface.block_texts()
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

    /// Type into block `i` of the surface (a block that is not blank).
    pub(super) fn blocks_type(&self, i: usize, at: usize, text: &str) {
        self.surface.type_at(i, at, text);
    }

    /// Press Enter in block `i` of the surface (a block that is not blank).
    pub(super) fn blocks_enter(&self, i: usize, at: usize) {
        self.surface.enter_at(i, at);
    }

    /// Is the block menu open?
    pub fn menu_open(&self) -> bool {
        self.surface.menu_open()
    }

    /// Press Enter at the end of the text and write a second block there (for pictures).
    pub fn press_enter_at_end(&self) {
        self.enter_at_end();
        self.type_in(usize::MAX, 0, "Next block");
    }

    fn enter_at_end(&self) {
        let end = self.surface.text().len();
        self.surface.put_cursor(usize::MAX, end);
        self.surface.enter_at_cursor();
    }

    /// A new empty block at the end with `/` typed into it: the block menu is open there (for pictures).
    pub fn open_menu_in_new_block(&self) {
        self.enter_at_end();
        self.type_in(usize::MAX, 0, "/");
    }

    /// Type into block `i` of the surface, as the keyboard would; a block that is not there is the empty one at the end.
    pub fn type_in(&self, i: usize, at: usize, text: &str) {
        self.surface.type_in_block(i, at, text);
    }

    /// How many blocks there are (the empty one at the end counts).
    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.surface.block_texts().len()
    }

    /// The block that has the keyboard, if any.
    pub fn focused(&self) -> Option<usize> {
        self.surface.focused_block()
    }

    /// Give the keyboard to block `i`, the cursor `column` bytes into it.
    pub fn put_cursor(&self, i: usize, column: usize) {
        self.surface.put_cursor(i, column);
    }

    /// The big title field above the surface (Enter in it moves on to the first block).
    pub fn title_view(&self, labels: &WikiLabels, report: &Report) -> Div {
        self.title.set_placeholder(&labels.title_hint);
        let report = Arc::clone(report);
        let (head, surface) = (self.head.clone(), Arc::clone(&self.surface));
        let field = self
            .title
            .clone()
            .on_change(move |text| {
                report(&assemble(&head, text, &split_blocks(&surface.text())));
            })
            .on_enter(jump_to(Arc::clone(&self.surface)));
        div()
            .class("fs-wk-b-title-field")
            .w_full()
            .child(field.build())
    }

    /// The writing surface. `report` gets the whole text after each change.
    pub fn view(&self, labels: &WikiLabels, report: &Report) -> Div {
        self.surface.view(
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
