//! The list screen (B): the lists at the left, a big title, a note, sections with a coloured heading, round boxes,
//! and a line at the bottom where a new task is typed and Enter makes it. No table, no column heads.
//!
//! Pattern: Adapter + Template method — the host's data in, one element out; the host decides every group and
//! every word (`Events`), the widget draws.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use blinc_layout::widgets::scroll::scroll;

use super::chip::Tone;
use super::entry::Entry;
use super::events::Events;
use super::line::TaskLine;
use super::side::{SideItem, SideList};

/// Width of the lists at the left.
const SIDE: f32 = 250.0;
/// The reading width of the main part; what is wider is white space.
const READING: f32 = 760.0;
/// Below this width the lists go away.
const NO_SIDE_BELOW: f32 = 760.0;

/// A group of lines under a coloured heading.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Section {
    /// The heading.
    pub title: String,
    /// The colour of the heading.
    pub tone: Tone,
    /// The lines.
    pub lines: Vec<TaskLine>,
}

impl Section {
    /// A section.
    pub fn new(title: &str, tone: Tone, lines: Vec<TaskLine>) -> Self {
        Self {
            title: title.into(),
            tone,
            lines,
        }
    }
}

/// Widths of the lists and of the reading column for a window `width` wide.
#[must_use]
pub fn list_layout(width: f32) -> (f32, f32) {
    let side = if width < NO_SIDE_BELOW { 0.0 } else { SIDE };
    let main = (width - side).max(0.0);
    (side, (main - 64.0).clamp(0.0, READING))
}

/// The data of the list screen.
pub struct ListScreen<'a> {
    /// The lists at the left, with headings for the groups.
    pub side: &'a [SideItem],
    /// The big title (the list chosen).
    pub title: &'a str,
    /// A note under the title.
    pub note: Option<&'a str>,
    /// The sections.
    pub sections: &'a [Section],
    /// What the entry line says while empty is the entry's own; this is the place it hands over (the list key).
    pub place: &'a str,
    /// The entry line.
    pub entry: &'a Entry,
    /// Said when there is nothing to show.
    pub empty: &'a str,
    /// The callbacks.
    pub events: &'a Events,
}

impl ListScreen<'_> {
    /// The element, filling `width` x `height`.
    pub fn build(&self, id: &str, width: f32, height: f32) -> Div {
        let (side, reading) = list_layout(width);
        let mut page = div()
            .class("fs-plan-page")
            .flex_col()
            .w(reading)
            .flex_shrink_0();
        page = page.child(text(self.title.to_string()).class("fs-plan-big-title"));
        if let Some(note) = self.note {
            page = page.child(text(note.to_string()).class("fs-plan-big-note"));
        }
        if self.sections.iter().all(|s| s.lines.is_empty()) {
            page = page.child(text(self.empty.to_string()).class("fs-plan-note"));
        }
        for section in self.sections.iter().filter(|s| !s.lines.is_empty()) {
            let mut block = div().class("fs-plan-section").flex_col().w_full();
            block = block.child(
                div().class("fs-plan-section-head").child(
                    text(format!("{}  {}", section.title, section.lines.len()))
                        .class("fs-plan-section-title")
                        .class(section.tone.class()),
                ),
            );
            for line in &section.lines {
                block = block.child(line.build(id, self.events));
            }
            page = page.child(block);
        }
        let add = div()
            .class("fs-plan-add")
            .flex_row()
            .w_full()
            .child(text("+").class("fs-plan-add-plus"))
            .child(
                self.entry
                    .build(&format!("{id}-entry"), self.place, self.events),
            );
        let main_w = (width - side).max(0.0);
        let main = div()
            .class("fs-plan-list-main")
            .flex_col()
            .w(main_w)
            .h(height)
            .min_h(0.0)
            .child(
                div()
                    .flex_1()
                    .min_h(0.0)
                    .w(main_w)
                    .child(scroll().w_full().h_full().content(page)),
            )
            .child(add);
        let mut root = div().class("fs-plan-root").flex_row().w(width).h(height);
        if side > 0.0 {
            root = root.child(SideList { items: self.side }.build(
                &format!("{id}-list"),
                side,
                self.events,
            ));
        }
        root.child(main)
    }
}
