//! The sortable list as elements.
//!
//! Pattern: Adapter — the state of the [`Sorter`](super::Sorter) in, Blinc layout out. The rows
//! are drawn from the state and clicking a row puts the cursor there; moving is not wired to the
//! pointer here (the frontend's gestures call `grab`, `hover`, `drop_grabbed`).

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::handle::{Labels as Words, SortableList};
use super::model::{Item, Lane, Pos};
use super::state::Sorter;
use super::window::window;
use crate::Element;
use crate::keys::bind_keys;

/// Draws an item in place of the plain label.
pub type Content = Arc<dyn Fn(&Item) -> Box<dyn Element> + Send + Sync>;

impl SortableList {
    /// Draw rows with `content` instead of the item's label.
    #[must_use]
    pub fn content(mut self, content: Content) -> Self {
        self.content = Some(content);
        self
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let this = self.clone();
        let body = self.live.view("lanes", move || this.render());
        let root = div().id(&self.id).class("fs-sl-root").w_full().child(body);
        match self.bindings.clone() {
            Some(_) => {
                let this = self.clone();
                bind_keys(
                    root,
                    Arc::new(move |event| this.press(&event) != super::model::Outcome::Ignored),
                )
            }
            None => root,
        }
    }

    fn render(&self) -> Div {
        self.read(|sorter| {
            let mut row = div().class("fs-sl");
            for (n, lane) in sorter.lanes().iter().enumerate() {
                row = row.child(self.lane(sorter, n, lane));
            }
            row
        })
    }

    fn lane(&self, sorter: &Sorter, n: usize, lane: &Lane) -> Div {
        let mut col = div().class("fs-sl-lane");
        if !lane.title.is_empty() {
            col = col.child(text(lane.title.clone()).class("fs-sl-title").no_wrap());
        }
        let focus = sorter.cursor().filter(|p| p.lane == n).map(|p| p.index);
        let rows = window(lane.items.len(), focus, self.max_rows);
        let words: &Words = &self.labels;
        if rows.start > 0 {
            let above = words.earlier.replacen("{}", &rows.start.to_string(), 1);
            col = col.child(text(above).class("fs-sl-note"));
        }
        for index in rows.clone() {
            col = col.child(self.row(sorter, lane, Pos { lane: n, index }));
        }
        let after = lane.items.len() - rows.end;
        if after > 0 {
            let below = words.later.replacen("{}", &after.to_string(), 1);
            col = col.child(text(below).class("fs-sl-note"));
        }
        if lane.items.is_empty() {
            col = col.child(text(words.empty.clone()).class("fs-sl-note"));
        }
        col
    }

    fn row(&self, sorter: &Sorter, lane: &Lane, pos: Pos) -> Div {
        let item = &lane.items[pos.index];
        let mut row = div()
            .class("fs-sl-row")
            .id(format!("{}-{}-{}", self.id, lane.key, item.key));
        if sorter.cursor() == Some(pos) {
            row = row.class("fs-sl-row-cursor");
        }
        if sorter.grabbed() == Some(item.key.as_str()) {
            row = row.class("fs-sl-row-grabbed");
        }
        if item.locked {
            row = row.class("fs-sl-row-locked");
        }
        row = match &self.content {
            Some(draw) => row.child_box(draw(item)),
            None => row.child(text(item.label.clone())),
        };
        let (this, key) = (self.clone(), item.key.clone());
        row.on_click(move |_| {
            this.set_cursor(&key);
        })
    }
}
