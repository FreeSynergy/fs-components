//! The field itself.
//!
//! Pattern: Adapter + Command — see the module `page_title`.

use std::sync::Arc;

use blinc_layout::div::div;
use fs_components_primitives::{FieldSize, SearchField};

use super::labels::TitleLabels;
use crate::Element;

/// Called when Enter is pressed in the title.
type OnEnter = Arc<dyn Fn() + Send + Sync>;

/// The title field. Clones share one field.
#[derive(Clone)]
pub struct PageTitle {
    field: SearchField,
    on_enter: Option<OnEnter>,
}

impl PageTitle {
    /// A title field with `text`, showing the placeholder of `labels` while empty.
    pub fn new(labels: &TitleLabels, text: &str) -> Self {
        let field = SearchField::new(&labels.placeholder).size(FieldSize::Title);
        // A title is not shown selected: typing adds to it.
        field.set_text_at_end(text);
        Self {
            field,
            on_enter: None,
        }
    }

    /// Show `text` instead of the placeholder given at `new` (a later language).
    pub fn set_placeholder(&self, text: &str) {
        self.field.set_placeholder(text);
    }

    /// Put `text` in the title, the cursor at its end. Does not call the change handler.
    pub fn set_text(&self, text: &str) {
        self.field.set_text_at_end(text);
    }

    /// Report every change of the text.
    #[must_use]
    pub fn on_change(mut self, handler: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.field = self.field.on_change(Arc::new(handler));
        self
    }

    /// Call `handler` on Enter in the title.
    #[must_use]
    pub fn on_enter(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_enter = Some(Arc::new(handler));
        self
    }

    /// The title now.
    pub fn text(&self) -> String {
        self.field.text()
    }

    /// Ask for the keyboard in the field.
    pub fn focus(&self) {
        self.field.focus();
    }

    /// Does the title have the keyboard?
    pub fn is_focused(&self) -> bool {
        self.field.is_focused()
    }

    /// The element to place above the content.
    pub fn build(&self) -> impl Element + use<> {
        let field = match self.on_enter.clone() {
            Some(handler) => self.field.clone().on_submit(handler),
            None => self.field.clone(),
        };
        div()
            .id("fs-page-title")
            .w_full()
            .class("fs-page-title")
            .child(field.build())
    }
}
