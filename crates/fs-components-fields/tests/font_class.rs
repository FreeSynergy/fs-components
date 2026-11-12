//! Every raw text input of the field components carries the font class, so the caret is measured in the font the
//! text is drawn in (see `fs-components-primitives` `with_field_font`).
//!
//! Pattern: none (tests). No graphics adapter: the tree of builders is walked, nothing is laid out.
#![allow(clippy::pedantic)]

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use blinc_core::reactive::global_graph;
use blinc_core::{BlincContextState, HookState};
use blinc_layout::prelude::ElementBuilder;
use fs_components_fields::{FieldInput, KeysAsText, Kind, Look};
use fs_components_primitives::FIELD_FONT_CLASS;
use fs_components_styles::StyleId;

fn count(b: &dyn ElementBuilder) -> usize {
    usize::from(b.element_classes().iter().any(|c| &**c == FIELD_FONT_CLASS))
        + b.children_builders()
            .iter()
            .map(|c| count(c.as_ref()))
            .sum::<usize>()
}

#[test]
fn every_part_input_of_a_field_has_the_font_class_named_or_not() {
    blinc_theme::ThemeState::init(blinc_cn::cn_bundle(), blinc_theme::ColorScheme::Light);
    if !BlincContextState::is_initialized() {
        BlincContextState::init(
            global_graph(),
            Arc::new(Mutex::new(HookState::new())),
            Arc::new(AtomicBool::new(false)),
        );
    }
    for (kind, parts) in [(Kind::Email, 2), (Kind::Phone, 2)] {
        for named in [false, true] {
            let mut field =
                FieldInput::new(kind, Look::new(StyleId::Admin, false), Arc::new(KeysAsText));
            if named {
                field = field.name("m");
            }
            let n = count(&field.build());
            // Red when `with_field_font` is taken out of `FieldInput::build`: 0 nodes carry the class.
            assert_eq!(n, parts, "{kind:?} named={named}: inputs with the class");
        }
    }
}
