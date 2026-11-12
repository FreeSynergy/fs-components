//! `FieldSize::Body`: the caret and the text must be measured and drawn in one font and size.
//!
//! Tests: the field that gets no class of its own is measured by Blinc in the default font while its text is drawn in
//! the font it inherits from the window (the caret then stands away from the last character); the class and its rule
//! in the style sheet are what prevent that, so they are held here. The measure on the screen (caret gap per block of
//! characters, at the level of the type components) is the typing test of `fs-files`.

use blinc_layout::prelude::ElementBuilder;
use blinc_theme::{ColorScheme, ThemeState};
use fs_components_primitives::{
    FIELD_FONT_CLASS, FieldSize, SEARCH_FIELD_STYLES, SearchField, with_field_font,
};

/// True if a node of the tree of `b` has the class `class`.
fn has_class(b: &dyn ElementBuilder, class: &str) -> bool {
    b.element_classes().iter().any(|c| &**c == class)
        || b.children_builders()
            .iter()
            .any(|c| has_class(c.as_ref(), class))
}

fn theme() {
    if ThemeState::try_get().is_none() {
        ThemeState::init(blinc_cn::cn_bundle(), ColorScheme::Light);
    }
}

#[test]
fn the_built_body_field_carries_the_class_and_the_class_has_a_rule() {
    theme();
    let built = SearchField::new("x").size(FieldSize::Body).build();
    // Red when `with_field_font` / `.class(..)` is taken out of `build()`: the node has no class, the caret is
    // measured in the default font (tried: with the line removed this assertion fails).
    assert!(
        has_class(&built, FIELD_FONT_CLASS),
        "no font class on the node"
    );
    assert!(has_class(&built, "fs-body-field"));
    assert!(SEARCH_FIELD_STYLES.contains(&format!(
        ".{FIELD_FONT_CLASS} {{ font-family: var(--font-sans)"
    )));
}

#[test]
fn the_cn_fields_get_their_font_from_the_cn_rule_and_the_class_it_names() {
    theme();
    for size in [FieldSize::Small, FieldSize::Medium, FieldSize::Large] {
        let built = SearchField::new("x").size(size).build();
        assert!(
            has_class(&built, "cn-input"),
            "{size:?}: no cn-input class on the node"
        );
    }
    // the rule that gives `.cn-input` the font is in the sheet that the style bundle always carries
    let sheet = blinc_cn::cn_styles::CN_STYLES;
    let at = sheet
        .find(".cn-input, .cn-textarea, .cn-label,")
        .expect("the cn rule list");
    let block = &sheet[at..at + sheet[at..].find('}').unwrap()];
    assert!(
        block.contains("font-family: var(--font-sans)"),
        ".cn-input has lost its font rule in blinc_cn"
    );
}

#[test]
fn a_raw_text_input_gets_the_class_only_through_the_one_helper() {
    theme();
    let data = fs_components_primitives::FieldData::new().text_input();
    let plain = blinc_layout::widgets::text_input::text_input(&data);
    assert!(
        !has_class(&plain, FIELD_FONT_CLASS),
        "counter: no class without the helper"
    );
    let with = with_field_font(blinc_layout::widgets::text_input::text_input(&data));
    assert!(has_class(&with, FIELD_FONT_CLASS));
}

#[test]
fn the_body_field_has_a_rule_that_gives_it_the_font_and_the_size_it_is_drawn_in() {
    assert!(SEARCH_FIELD_STYLES.contains(".fs-body-field"));
    assert!(SEARCH_FIELD_STYLES.contains("font-family: var(--font-sans)"));
    assert!(SEARCH_FIELD_STYLES.contains("font-size: 14px"));
}
