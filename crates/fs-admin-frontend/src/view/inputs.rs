//! The inputs of a form: the component of the type where `fs-components-fields` has one, a plain input where not.
//!
//! Pattern: Adapter over Blinc's `text_input` for the plain input (same measures and colours as the field
//! components, from the [`Look`]), Strategy by basic type for the choice between them. The plain input is a stopgap
//! that belongs in `fs-components-fields` (ADR 0066 point 4, one component per basic type, in that crate); it is here
//! until Thread C builds it, and `DESIGN.md` says so.

use std::sync::Arc;

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::div;
use blinc_layout::widgets::text_input::{SharedTextInputData, text_input};
use fs_components_fields::{FieldInput, Look, Say, Suggest};
use fs_components_primitives::FieldData;
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
    /// Keeps the signals of the input alive as long as the slot lives (not as long as the page build that made it).
    _keep: FieldData,
}

impl Slot {
    /// A slot for `field`, starting with `text`, reporting changes to `on`.
    pub fn new(
        look: Look,
        say: &Arc<dyn Say>,
        field: &FieldDescription,
        text: &str,
        id: &str,
        suggest: Option<&Arc<dyn Suggest>>,
        on: OnText,
    ) -> Self {
        let keep = FieldData::new();
        let body = Self::body(look, say, field, (text, id), suggest, &on, &keep);
        Self {
            body,
            on,
            _keep: keep,
        }
    }

    fn body(
        look: Look,
        say: &Arc<dyn Say>,
        field: &FieldDescription,
        (text, id): (&str, &str),
        suggest: Option<&Arc<dyn Suggest>>,
        on: &OnText,
        keep: &FieldData,
    ) -> Body {
        if let Some(kind) = kind_of(field.basic) {
            let on = Arc::clone(on);
            let mut input = FieldInput::new(kind, look, Arc::clone(say))
                .value(text)
                .name(id)
                .on_change(move |t| on(t));
            if let Some(s) = suggest {
                input = input.suggest(Arc::clone(s));
            }
            return Body::Typed(input);
        }
        if field.basic == BasicType::Choice {
            return Body::Choice;
        }
        let data = keep.text_input();
        {
            let mut d = data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            d.cursor = text.chars().count();
            text.clone_into(&mut d.value);
        }
        Body::Plain(data)
    }
}

/// The plain input element; `on` hears every change.
pub fn plain(
    look: &Look,
    say: &dyn Say,
    field: &FieldDescription,
    data: &SharedTextInputData,
    name: &str,
    on: OnText,
) -> impl Element + use<> {
    let (s, pal) = (look.style, look.palette());
    fs_components_primitives::with_field_font(text_input(data))
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
pub fn choice(
    look: &Look,
    say: &dyn Say,
    field: &FieldDescription,
    chosen: &str,
    name: &str,
    pick: &OnText,
) -> impl Element + use<> {
    let s = look.style;
    let mut row = div().flex_row().flex_wrap().gap_px(s.spacing.xs).w_full();
    for key in &field.choices {
        let (k, on) = (key.clone(), Arc::clone(pick));
        row = row.child(cn_button(
            &say.say(&format!("{}.{key}", field.label)),
            if key == chosen {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Secondary
            },
            &format!("{name}-{key}"),
            move || on(&k),
        ));
    }
    row
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use blinc_core::owner::BuildScope;
    use blinc_core::reactive::global_graph;
    use fs_components_fields::{KeysAsText, Look, Say};
    use fs_components_styles::StyleId;
    use fs_types::{BasicType, FieldDescription};

    use super::{Body, OnText, Slot};

    fn live() -> usize {
        global_graph().lock().unwrap().stats().signal_count
    }

    /// Fall C at the slot: a slot is made while a page is built; the page is built again and the scope of the first
    /// build is disposed; the slot lives on (it is cached) and so must the signals of its data. Red when the data is
    /// made with Blinc's constructors directly (the scope takes the signals with it).
    #[test]
    fn the_signals_of_a_slot_outlive_the_page_build_that_made_it_and_go_with_the_slot() {
        let _g = crate::view::graph_test_lock();
        let say: Arc<dyn Say> = Arc::new(KeysAsText);
        let look = Look::new(StyleId::Admin, false);
        let on: OnText = Arc::new(|_| {});
        for (basic, plain) in [(BasicType::MultilineText, true), (BasicType::Email, false)] {
            let field = FieldDescription::new("f", basic).label("f");
            fp_collect();
            let base = live();
            let mut scope = BuildScope::new();
            let pass = scope.begin();
            let slot = Slot::new(look, &say, &field, "x", "f-0", None, Arc::clone(&on));
            scope.finish(pass);
            assert_eq!(matches!(slot.body, Body::Plain(_)), plain, "{basic:?}");
            let made = live();
            assert!(made > base, "{basic:?}: the slot makes signals");
            let pass = scope.begin();
            scope.finish(pass);
            assert_eq!(
                live(),
                made,
                "{basic:?}: the signals died with the build that made the slot"
            );
            for _ in 0..1000 {
                let pass = scope.begin();
                scope.finish(pass);
            }
            assert_eq!(live(), made, "{basic:?}: constant over 1000 page builds");
            drop(slot);
            fp_collect();
            assert_eq!(live(), base, "{basic:?}: the slot gives its signals back");
        }
    }

    /// The plain input has the font class, so its caret is measured in the font its text is drawn in. Red when
    /// `with_field_font` is taken out of `plain`.
    #[test]
    fn the_plain_input_carries_the_font_class() {
        use blinc_layout::prelude::ElementBuilder;
        let _g = crate::view::graph_test_lock();
        blinc_theme::ThemeState::init(blinc_cn::cn_bundle(), blinc_theme::ColorScheme::Light);
        if !blinc_core::BlincContextState::is_initialized() {
            blinc_core::BlincContextState::init(
                global_graph(),
                Arc::new(std::sync::Mutex::new(blinc_core::HookState::new())),
                Arc::new(std::sync::atomic::AtomicBool::new(false)),
            );
        }
        let say = KeysAsText;
        let look = Look::new(StyleId::Admin, false);
        let field = FieldDescription::new("f", BasicType::Text).label("f");
        let keep = fs_components_primitives::FieldData::new();
        let data = keep.text_input();
        let on: OnText = Arc::new(|_| {});
        let built = super::plain(&look, &say, &field, &data, "f-0", on);
        assert!(
            built
                .element_classes()
                .iter()
                .any(|c| &**c == fs_components_primitives::FIELD_FONT_CLASS),
            "no font class on the plain input"
        );
    }

    fn fp_collect() {
        fs_components_primitives::collect_field_data();
    }
}
