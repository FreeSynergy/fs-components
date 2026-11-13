//! `search-field`: a text field that reports what is typed.
//!
//! Pattern: Adapter over `blinc_cn`'s input (not rebuilt). The field keeps its text in shared
//! state, so a clone of [`SearchField`] kept by the shell survives rebuilds of the layout.

use std::sync::Arc;

use blinc_cn::components::input::{InputSize, input};
use blinc_layout::div::div;
use blinc_layout::widgets::text_input::{SharedTextInputData, TextInput, text_input};
use blinc_theme::{ColorToken, ThemeState};

use crate::field_data::FieldData;

/// The size of a page title in pixels, written ONCE: the macro gives the literal to the style sheet (`concat!`) and to
/// [`TITLE_PX`]. Both the title field and the reading title of the wiki frontends use it.
macro_rules! title_px {
    () => {
        40
    };
}

/// The text size of a page title (the field of [`FieldSize::Title`] and the reading title above a page).
#[allow(clippy::unnecessary_cast)] // the literal of the macro is an integer (it is also a piece of CSS)
pub const TITLE_PX: f32 = title_px!() as f32;

/// The style sheet of the field. The caret of Blinc's `text_input` is placed from the font the stylesheet gives the
/// FIELD (only an element with a class or an id is looked up), while the text is drawn in the font it inherits from the
/// window: a field with no class of its own measured in one font and drew in another, and the caret stood away from
/// the last character (measured 2026-11-12: 4 to 15 px, against 0 to 2 px in the fields of the type components). The
/// class and this rule give the field the same font and size for both.
pub const STYLES: &str = concat!(
    ".fs-field-font { font-family: var(--font-sans); }\n.fs-body-field { font-family: var(--font-sans); font-size: 14px; }\n",
    ".fs-title-field, .fs-title-field:hover, .fs-title-field:focus { font-family: var(--font-sans); font-size: ",
    title_px!(),
    "px; }\n"
);

/// The class that gives a raw `text_input` (one that is not Blinc's `input` of `blinc_cn`, whose `.cn-input` rule
/// already names the font) the font the caret is measured in. The ONE place of the rule: every raw `text_input` of
/// these crates goes through [`with_field_font`].
pub const FIELD_FONT_CLASS: &str = "fs-field-font";

/// Put `field` in the font of the window for the measure of the caret (class [`FIELD_FONT_CLASS`], rule in
/// [`STYLES`]). Pattern: none (pure function over a builder).
#[must_use]
pub fn with_field_font(field: TextInput) -> TextInput {
    field.class(FIELD_FONT_CLASS)
}

/// The size of [`FieldSize::Body`].
const BODY_TEXT: f32 = 14.0;
const BODY_HEIGHT: f32 = 36.0;
/// The field is this many times as high as its text.
const TITLE_LINE: f32 = 1.4;

/// Called with the whole text after every change.
pub type OnChange = Arc<dyn Fn(&str) + Send + Sync>;

/// Called when Enter is pressed in the field.
pub type OnSubmit = Arc<dyn Fn() + Send + Sync>;

/// How much room the field takes. The real height comes from the theme.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldSize {
    /// For lists and narrow bars.
    Small,
    /// The default.
    #[default]
    Medium,
    /// For a field that is the main thing on the screen.
    Large,
    /// Text of exactly body size (14 px), as the fields of the components of the types have it, in a
    /// control 36 px high: what a typed word must look like to be read and tested at one size.
    Body,
    /// A page title: big text, no frame and no fill, as the title of a page in Notion. The field is the title.
    Title,
}

/// A search field.
#[derive(Clone)]
pub struct SearchField {
    data: SharedTextInputData,
    /// Gives the signals of `data` back when the field is gone.
    _keep: FieldData,
    on_change: Option<OnChange>,
    on_submit: Option<OnSubmit>,
    size: FieldSize,
}

impl SearchField {
    /// A field showing `placeholder` while empty.
    pub fn new(placeholder: &str) -> Self {
        let keep = FieldData::new();
        Self {
            data: keep.text_input_with_placeholder(placeholder),
            _keep: keep,
            on_change: None,
            on_submit: None,
            size: FieldSize::default(),
        }
    }

    /// Take this much room instead of the default.
    #[must_use]
    pub fn size(mut self, size: FieldSize) -> Self {
        self.size = size;
        self
    }

    /// What is typed now.
    pub fn text(&self) -> String {
        self.data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .value
            .clone()
    }

    /// Put `text` in the field with all of it selected (typing replaces it, an arrow key keeps it), as a field opens
    /// with a name to change. Does not call the change handler: the caller set it. The undo history is emptied, so
    /// Ctrl+Z cannot bring back an older text; the scroll position goes back to the start.
    pub fn set_text(&self, text: &str) {
        self.put(text, true);
    }

    /// As [`SearchField::set_text`], but nothing is selected and the cursor stands at the end (a title that is edited,
    /// not replaced).
    pub fn set_text_at_end(&self, text: &str) {
        self.put(text, false);
    }

    fn put(&self, text: &str, select: bool) {
        let mut d = self
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Blinc keeps both stacks private. Undo everything (undo stack empty), then one edit clears the redo stack;
        // its single undo entry holds the very text set below, so Ctrl+Z changes nothing.
        while d.undo() {}
        text.clone_into(&mut d.value);
        d.move_to_end(false);
        d.insert("-");
        text.clone_into(&mut d.value);
        d.scroll_offset_x = 0.0;
        d.select_all();
        if !select {
            d.move_to_end(false);
        }
    }

    /// Does the field have the keyboard now?
    pub fn is_focused(&self) -> bool {
        use blinc_layout::stateful::TextFieldState::{Focused, FocusedHovered};
        matches!(
            self.data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .visual,
            Focused | FocusedHovered
        )
    }

    /// Show `text` instead of the placeholder given to [`SearchField::new`] while the field is empty.
    pub fn set_placeholder(&self, text: &str) {
        text.clone_into(
            &mut self
                .data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .placeholder,
        );
    }

    /// Give the keyboard to the field on the next frame (after it is in the tree).
    pub fn focus(&self) {
        blinc_layout::widgets::text_input::focus_text_input_deferred(&self.data);
    }

    /// Report every change of the text.
    #[must_use]
    pub fn on_change(mut self, handler: OnChange) -> Self {
        self.on_change = Some(handler);
        self
    }

    /// Call `handler` when Enter is pressed in the field (a single-line field does nothing on Enter otherwise).
    #[must_use]
    pub fn on_submit(mut self, handler: OnSubmit) -> Self {
        self.on_submit = Some(handler);
        self
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl blinc_layout::prelude::ElementBuilder + use<> {
        let size = match self.size {
            FieldSize::Small => InputSize::Small,
            FieldSize::Large => InputSize::Large,
            _ => InputSize::Medium,
        };
        let handler = self.on_change.clone();
        if self.size == FieldSize::Title {
            let theme = ThemeState::get();
            let none = blinc_core::Color::TRANSPARENT;
            let mut field = with_field_font(text_input(&self.data))
                .h(TITLE_PX * TITLE_LINE)
                .text_size(TITLE_PX)
                .padding_x(0.0)
                .border(0.0, none)
                .idle_bg_color(none)
                .text_color(theme.color(ColorToken::TextPrimary))
                .placeholder_color(theme.color(ColorToken::TextTertiary))
                .class("fs-title-field")
                .w_full();
            if let Some(handler) = handler {
                field = field.on_change(move |text| handler(text));
            }
            if let Some(submit) = self.on_submit.clone() {
                field = field.on_submit(move |_| submit());
            }
            return div().w_full().child(field);
        }
        if self.size == FieldSize::Body {
            let theme = ThemeState::get();
            let mut field = with_field_font(text_input(&self.data))
                .h(BODY_HEIGHT)
                .text_size(BODY_TEXT)
                .rounded(6.0)
                .border(1.0, theme.color(ColorToken::Border))
                .idle_bg_color(theme.color(ColorToken::InputBg))
                .text_color(theme.color(ColorToken::TextPrimary))
                .placeholder_color(theme.color(ColorToken::TextTertiary))
                .class("fs-body-field")
                .w_full();
            if let Some(handler) = handler {
                field = field.on_change(move |text| handler(text));
            }
            return div().w_full().child(field);
        }
        let mut field = input(&self.data).size(size).w_full();
        if let Some(handler) = handler {
            field = field.on_change(move |text| handler(text));
        }
        div().w_full().child(field)
    }
}

#[cfg(test)]
mod tests {
    use super::SearchField;

    #[test]
    fn set_text_forgets_the_undo_history() {
        let field = SearchField::new("name");
        {
            let mut d = field.data.lock().unwrap();
            d.insert("old");
            d.insert(" text");
        }
        field.set_text("new");
        let mut d = field.data.lock().unwrap();
        // Redo has nothing; undo may step, but never to an older text.
        assert!(!d.redo());
        d.undo();
        assert_eq!(d.value, "new");
        assert!(!d.undo());
    }

    #[test]
    fn set_text_without_history_still_sets() {
        let field = SearchField::new("name");
        field.set_text("李雷 ß");
        assert_eq!(field.text(), "李雷 ß");
    }
}
