//! The list at the side: projects (board) or lists and areas (day list), each with a count.
//!
//! Pattern: Adapter — items in, an element out; an item with no `key` is a heading of a group of items.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::events::{Events, tell};

/// One line of the side list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SideItem {
    /// Names it for the host; empty for a heading.
    pub key: String,
    /// The text.
    pub title: String,
    /// How many; `None` shows nothing.
    pub count: Option<usize>,
    /// Drawn as the one chosen.
    pub selected: bool,
}

impl SideItem {
    /// A line that can be chosen.
    pub fn new(key: &str, title: &str, count: Option<usize>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            count,
            selected: false,
        }
    }

    /// A heading of a group.
    pub fn heading(title: &str) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }
}

/// The side list.
pub struct SideList<'a> {
    /// The lines.
    pub items: &'a [SideItem],
}

impl SideList<'_> {
    /// The element, `width` wide.
    pub fn build(&self, id: &str, width: f32, events: &Events) -> Div {
        let mut col = div()
            .class("fs-plan-side")
            .flex_col()
            .w(width)
            .flex_shrink_0();
        for item in self.items {
            if item.key.is_empty() {
                col = col.child(text(item.title.clone()).class("fs-plan-side-heading"));
                continue;
            }
            let mut line = div()
                .class("fs-plan-side-item")
                .flex_row()
                .w_full()
                .id(format!("{id}-{}", item.key));
            if item.selected {
                line = line.class("fs-plan-side-selected");
            }
            line = line
                .child(text(item.title.clone()).class("fs-plan-side-title"))
                .child(div().flex_1());
            if let Some(n) = item.count {
                line = line.child(text(n.to_string()).class("fs-plan-side-count"));
            }
            let (pick, key) = (events.pick.clone(), item.key.clone());
            col = col.child(line.on_click(move |_| tell(&pick, &key)));
        }
        col
    }
}
