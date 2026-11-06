//! `search-field`: a text field that reports what is typed.
//!
//! Pattern: Adapter over `blinc_cn`'s input (not rebuilt). The field keeps its text in shared
//! state, so a clone of [`SearchField`] kept by the shell survives rebuilds of the layout.

use std::sync::Arc;

use blinc_cn::components::input::input;
use blinc_layout::widgets::text_input::{SharedTextInputData, text_input_data_with_placeholder};

/// Called with the whole text after every change.
pub type OnChange = Arc<dyn Fn(&str) + Send + Sync>;

/// A search field.
#[derive(Clone)]
pub struct SearchField {
    data: SharedTextInputData,
    on_change: Option<OnChange>,
}

impl SearchField {
    /// A field showing `placeholder` while empty.
    pub fn new(placeholder: &str) -> Self {
        Self {
            data: text_input_data_with_placeholder(placeholder),
            on_change: None,
        }
    }

    /// Report every change of the text.
    #[must_use]
    pub fn on_change(mut self, handler: OnChange) -> Self {
        self.on_change = Some(handler);
        self
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl blinc_layout::prelude::ElementBuilder + use<> {
        let mut field = input(&self.data).w_full();
        if let Some(handler) = &self.on_change {
            let handler = Arc::clone(handler);
            field = field.on_change(move |text| handler(text));
        }
        field
    }
}
