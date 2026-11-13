//! The dialog and its commands.
//!
//! Pattern: Command + Adapter — `confirm` and `cancel` are what the buttons and the keys (Enter, Escape) call; the text
//! lives in the field's own state, outside anything that is drawn again, so typing never loses the keyboard.

use std::sync::Arc;

use blinc_cn::components::button::{ButtonBuilder, ButtonVariant};
use blinc_layout::div::div;
use blinc_layout::prelude::text;
use fs_components_primitives::{FieldSize, SearchField};

use super::labels::Labels;
use crate::Element;

/// What the name is for; it decides the title.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    /// A new page.
    NewPage,
    /// A page below another one (`about` = the parent).
    SubPage,
    /// A new name for a page (`about` = the old name).
    Rename,
}

/// Called with the trimmed name when the person confirms.
pub type OnName = Arc<dyn Fn(&str) + Send + Sync>;
/// Called when the person leaves without a name.
pub type OnCancel = Arc<dyn Fn() + Send + Sync>;

/// The dialog. Clones share one field and one pair of callbacks. The frontend puts [`NameDialog::build`] in its modal.
#[derive(Clone)]
pub struct NameDialog {
    id: String,
    title: String,
    labels: Labels,
    field: SearchField,
    on_name: Option<OnName>,
    on_cancel: Option<OnCancel>,
}

impl NameDialog {
    /// A dialog for `purpose`; `about` fills the `{}` of the title (the parent page, the old name).
    pub fn new(labels: &Labels, purpose: Purpose, about: &str) -> Self {
        let title = match purpose {
            Purpose::NewPage => labels.new_page.clone(),
            Purpose::SubPage => labels.sub_page.replacen("{}", about, 1),
            Purpose::Rename => labels.rename.replacen("{}", about, 1),
        };
        Self {
            id: "name-dialog".to_owned(),
            title,
            labels: labels.clone(),
            field: SearchField::new(&labels.placeholder).size(FieldSize::Body),
            on_name: None,
            on_cancel: None,
        }
    }

    /// The prefix of the ids of the parts (several dialogs in one window).
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// Start with `name` in the field, all of it selected.
    #[must_use]
    pub fn initial(self, name: &str) -> Self {
        self.field.set_text(name);
        self
    }

    /// Call `handler` with the trimmed name on OK or Enter.
    #[must_use]
    pub fn on_name(mut self, handler: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_name = Some(Arc::new(handler));
        self
    }

    /// Call `handler` on Cancel or Escape.
    #[must_use]
    pub fn on_cancel(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancel = Some(Arc::new(handler));
        self
    }

    /// The title shown.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// What is typed now.
    pub fn text(&self) -> String {
        self.field.text()
    }

    /// Ask for the keyboard in the field (call when the dialog is shown).
    pub fn focus(&self) {
        self.field.focus();
    }

    /// Take the name: false (nothing happens, the dialog stays) while the field holds only blanks.
    pub fn confirm(&self) -> bool {
        let name = self.field.text();
        let name = name.trim();
        if name.is_empty() {
            return false;
        }
        if let Some(handler) = &self.on_name {
            handler(name);
        }
        true
    }

    /// Leave without a name.
    pub fn cancel(&self) {
        if let Some(handler) = &self.on_cancel {
            handler();
        }
    }

    /// The element to place in a modal.
    pub fn build(&self) -> impl Element + use<> {
        let (ok, cancel) = (self.clone(), self.clone());
        let keys = self.clone();
        let button = |tag: &str, word: &str, variant| {
            let id = format!("{}-{tag}", self.id);
            ButtonBuilder::with_key(id.clone(), word.to_owned()).variant(variant)
        };
        let buttons = div()
            .class("fs-nd-buttons")
            .child(
                button("cancel", &self.labels.cancel, ButtonVariant::Outline)
                    .on_click(move |_| cancel.cancel())
                    .build_component()
                    .id(&format!("{}-cancel", self.id)),
            )
            .child(
                button("ok", &self.labels.ok, ButtonVariant::Primary)
                    .on_click(move |_| {
                        ok.confirm();
                    })
                    .build_component()
                    .id(&format!("{}-ok", self.id)),
            );
        div()
            .id(&self.id)
            .class("fs-nd")
            .child(text(self.title.clone()).class("fs-nd-title"))
            .child(
                div()
                    .id(format!("{}-field", self.id))
                    .child(self.field.build()),
            )
            .child(buttons)
            // The field keeps the keyboard while typing; Enter and Escape reach the dialog as broadcast keys.
            .on_key_down(move |ctx| match ctx.key_code {
                13 => {
                    keys.confirm();
                }
                27 => keys.cancel(),
                _ => {}
            })
    }
}
