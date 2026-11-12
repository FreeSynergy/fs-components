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

/// The style sheet of the field. The caret of Blinc's `text_input` is placed from the font the stylesheet gives the
/// FIELD (only an element with a class or an id is looked up), while the text is drawn in the font it inherits from the
/// window: a field with no class of its own measured in one font and drew in another, and the caret stood away from
/// the last character (measured 2026-11-12: 4 to 15 px, against 0 to 2 px in the fields of the type components). The
/// class and this rule give the field the same font and size for both.
pub const STYLES: &str =
    ".fs-field-font { font-family: var(--font-sans); }\n.fs-body-field { font-family: var(--font-sans); font-size: 14px; }\n";

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

/// Called with the whole text after every change.
pub type OnChange = Arc<dyn Fn(&str) + Send + Sync>;

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
}

/// A search field.
#[derive(Clone)]
pub struct SearchField {
    data: SharedTextInputData,
    /// Gives the signals of `data` back when the field is gone.
    _keep: FieldData,
    on_change: Option<OnChange>,
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
            size: FieldSize::default(),
        }
    }

    /// Take this much room instead of the default.
    #[must_use]
    pub fn size(mut self, size: FieldSize) -> Self {
        self.size = size;
        self
    }

    /// Report every change of the text.
    #[must_use]
    pub fn on_change(mut self, handler: OnChange) -> Self {
        self.on_change = Some(handler);
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
