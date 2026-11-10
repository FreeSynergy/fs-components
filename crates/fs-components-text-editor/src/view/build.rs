//! The editor as one element: toolbar, text area, preview, counters.
//!
//! Pattern: View function — a pure function from the handle's state to elements; the text area is
//! the one part that is not drawn again.

use blinc_cn::components::textarea::textarea;
use blinc_layout::div::div;
use blinc_layout::prelude::text;
use fs_components_widgets::{Element, TextView};

use super::handle::EditorView;
use crate::editor::TextEditor;

impl EditorView {
    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let (bar, preview, status) = (self.clone(), self.clone(), self.clone());
        let typed = self.clone();
        let field = textarea(&self.area)
            .w_full()
            .rows(10)
            .placeholder(self.labels.placeholder.clone())
            .on_change(move |value| typed.typed(value));
        div()
            .flex_col()
            .gap(8.0)
            .w_full()
            .id(&self.id)
            .child(self.live.region("bar", move || bar.toolbar()))
            .child(field)
            .child(self.live.region("preview", move || {
                let blocks = (preview.previewer)(&preview.text());
                TextView::new(blocks).build()
            }))
            .child(self.live.region("status", move || status.status()))
    }

    fn status(&self) -> blinc_layout::div::Div {
        let c = self.counts();
        let line = self
            .labels
            .counts
            .replacen("{chars}", &c.chars.to_string(), 1)
            .replacen("{words}", &c.words.to_string(), 1);
        let mut row = div()
            .flex_row()
            .gap(12.0)
            .child(text(line).id(format!("{}-counts", self.id)));
        if self.is_dirty() {
            row = row.child(text(self.labels.unsaved.clone()).id(format!("{}-unsaved", self.id)));
        }
        row
    }
}
