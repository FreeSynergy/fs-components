//! The entry line: one line of text; Enter hands it to the host and empties the line.
//!
//! Pattern: Adapter over Blinc's `text_input` (the same field, the same font rule, as the search field); the
//! one thing new is `on_submit`. The text a host receives is trimmed and never empty; a line of blanks is kept as it
//! is. The host keeps one `Entry` per place and builds it again on every drawing, so what is typed survives.

use blinc_layout::div::{Div, div};
use blinc_layout::widgets::text_input::{SharedTextInputData, text_input};
use fs_components_primitives::{FieldData, with_field_font};

use super::events::{Events, redraw};

/// One entry line.
#[derive(Clone)]
pub struct Entry {
    data: SharedTextInputData,
    _keep: FieldData,
}

/// What Enter hands over: the trimmed text, or `None` for a line with no letter in it.
pub fn submitted(typed: &str) -> Option<String> {
    let text = typed.trim();
    (!text.is_empty()).then(|| text.to_string())
}

impl Entry {
    /// A line showing `placeholder` while empty.
    pub fn new(placeholder: &str) -> Self {
        let keep = FieldData::new();
        Self {
            data: keep.text_input_with_placeholder(placeholder),
            _keep: keep,
        }
    }

    /// What is typed now.
    pub fn text(&self) -> String {
        self.data
            .lock()
            .map(|d| d.value.clone())
            .unwrap_or_default()
    }

    /// Empty the line.
    pub fn clear(&self) {
        if let Ok(mut d) = self.data.lock() {
            d.value.clear();
            d.cursor = 0;
            d.selection_start = None;
            d.force_sync_once = true;
        }
    }

    /// The element; `place` is handed to the host with the text.
    pub fn build(&self, id: &str, place: &str, events: &Events) -> Div {
        let (me, enter, place) = (self.clone(), events.enter.clone(), place.to_string());
        let field = with_field_font(text_input(&self.data))
            .class("fs-plan-entry-field")
            .w_full()
            .on_submit(move |_| {
                let Some(text) = submitted(&me.text()) else {
                    return;
                };
                me.clear();
                if let Some(enter) = &enter {
                    enter(&place, &text);
                }
                redraw();
            });
        div()
            .id(id.to_string())
            .class("fs-plan-entry")
            .w_full()
            .child(field)
    }
}
