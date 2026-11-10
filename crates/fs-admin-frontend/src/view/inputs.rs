//! The inputs of a form: the component of the type where `fs-components-fields` has one, a plain input where not.
//!
//! Pattern: Adapter over Blinc's `text_input` for the plain input (same measures and colours as the field
//! components, from the [`Look`]), Strategy by basic type for the choice between them. The plain input is a stopgap
//! that belongs in `fs-components-fields` (ADR 0066 point 4, one component per basic type, in that crate); it is here
//! until Thread C builds it, and `DESIGN.md` says so.

use std::sync::Arc;

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::div;
use blinc_layout::widgets::text_input::{SharedTextInputData, text_input, text_input_data};
use fs_components_fields::{FieldInput, Look, Say, Suggest};
use fs_components_widgets::Element;
use fs_types::{BasicType, FieldDescription};

use super::chrome::cn_button;
use super::kinds::kind_of;

/// Called with the text of the slot after every change.
pub type OnText = Arc<dyn Fn(&str) + Send + Sync>;

/// What a slot is made of.
#[derive(Clone)]
pub enum Body {
    /// The component of `fs-components-fields`.
    Typed(FieldInput),
    /// One line of text.
    Plain(SharedTextInputData),
    /// One of the keys of the field's list; the text is the chosen key.
    Choice,
}

/// A slot of a form, made once and kept while the form is open.
#[derive(Clone)]
pub struct Slot {
    /// The input.
    pub body: Body,
    /// Hears the text after every change.
    pub on: OnText,
}

impl Slot {
    /// A slot for `field`, starting with `text`, reporting changes to `on`.
    pub fn new(
        look: Look,
        say: &Arc<dyn Say>,
        field: &FieldDescription,
        text: &str,
        suggest: Option<&Arc<dyn Suggest>>,
        on: OnText,
    ) -> Self {
        let body = Self::body(look, say, field, text, suggest, &on);
        Self { body, on }
    }

    fn body(
        look: Look,
        say: &Arc<dyn Say>,
        field: &FieldDescription,
        text: &str,
        suggest: Option<&Arc<dyn Suggest>>,
        on: &OnText,
    ) -> Body {
        if let Some(kind) = kind_of(field.basic) {
            let on = Arc::clone(on);
            let mut input = FieldInput::new(kind, look, Arc::clone(say))
                .value(text)
                .on_change(move |t| on(t));
            if let Some(s) = suggest {
                input = input.suggest(Arc::clone(s));
            }
            return Body::Typed(input);
        }
        if field.basic == BasicType::Choice {
            return Body::Choice;
        }
        let data = text_input_data();
        {
            let mut d = data.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            d.cursor = text.chars().count();
            text.clone_into(&mut d.value);
        }
        Body::Plain(data)
    }
}

/// The plain input element; `on` hears every change.
pub fn plain(look: &Look, say: &dyn Say, field: &FieldDescription, data: &SharedTextInputData, name: &str, on: OnText) -> impl Element + use<> {
    let (s, pal) = (look.style, look.palette());
    text_input(data)
        .id(name)
        .h(s.heights.control)
        .text_size(s.text.body)
        .rounded(s.radii.control)
        .border(1.0, Look::color(pal.border))
        .idle_bg_color(Look::color(pal.field))
        .text_color(Look::color(pal.text))
        .placeholder_color(Look::color(pal.text_weak))
        .cursor_color(Look::color(pal.accent))
        .selection_color(Look::color(pal.selected))
        .placeholder(say.say(&format!("hint.{}", field.label)))
        .w_full()
        .on_change(move |t| on(t))
}

/// The choice as a row of buttons; the chosen one is marked. `pick` hears the key.
pub fn choice(look: &Look, say: &dyn Say, field: &FieldDescription, chosen: &str, name: &str, pick: &OnText) -> impl Element + use<> {
    let s = look.style;
    let mut row = div().flex_row().flex_wrap().gap_px(s.spacing.xs).w_full();
    for key in &field.choices {
        let (k, on) = (key.clone(), Arc::clone(pick));
        row = row.child(
            cn_button(
                &say.say(&format!("{}.{key}", field.label)),
                if key == chosen { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                &format!("{name}-{key}"),
                move || on(&k),
            ),
        );
    }
    row
}
