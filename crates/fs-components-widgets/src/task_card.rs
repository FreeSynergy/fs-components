//! `task-card`: one task as a small card, for a column or a list.
//!
//! Pattern: Value object + Adapter — the data is what the caller says (a title, lines, tags,
//! marks), neutral: the widget has no words and no colours of its own, and does not know what a
//! task is. It is a plain element with no click of its own, so it may be drawn inside a live area
//! and inside a row that already takes clicks.

use blinc_layout::div::div;
use blinc_layout::prelude::text;

use crate::Element;

/// One card.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskCard {
    /// The main text.
    pub title: String,
    /// Smaller lines below it, in the caller's words.
    pub lines: Vec<String>,
    /// Short tags.
    pub tags: Vec<String>,
    /// Markings that must be seen at once, in the caller's words.
    pub marks: Vec<String>,
    /// Drawn as the card that is open.
    pub open: bool,
}

impl TaskCard {
    /// A card with this title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let mut card = div().class("fs-task-card").flex_col().w_full();
        if self.open {
            card = card.class("fs-task-card-open");
        }
        if !self.marks.is_empty() {
            let mut row = div().class("fs-task-card-tags").flex_row().flex_wrap();
            for mark in &self.marks {
                row = row.child(div().class("fs-task-card-mark").child(text(mark.clone())));
            }
            card = card.child(row);
        }
        card = card.child(text(self.title.clone()).class("fs-task-card-title"));
        for line in &self.lines {
            card = card.child(text(line.clone()).class("fs-task-card-line"));
        }
        if !self.tags.is_empty() {
            let mut row = div().class("fs-task-card-tags").flex_row().flex_wrap();
            for tag in &self.tags {
                row = row.child(div().class("fs-task-card-tag").child(text(tag.clone())));
            }
            card = card.child(row);
        }
        card
    }
}

/// CSS of the card: only theme variables.
pub const STYLES: &str = r"
.fs-task-card { min-width: 0; gap: var(--space-1); }
.fs-task-card-title { color: var(--text-primary); font-size: var(--text-sm); font-weight: 600; }
.fs-task-card-open .fs-task-card-title { color: var(--primary); }
.fs-task-card-line { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-task-card-tags { display: flex; flex-direction: row; flex-wrap: wrap; gap: var(--space-1); }
.fs-task-card-tag { padding: 0 var(--space-1); border-radius: var(--radius-default); background: var(--accent-subtle); color: var(--text-secondary); font-size: var(--text-xs); }
.fs-task-card-mark { padding: 0 var(--space-1); border-radius: var(--radius-default); background: var(--warning-bg); color: var(--text-primary); font-size: var(--text-xs); }
";
