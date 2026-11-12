//! `FieldSize::Body`: the caret and the text must be measured and drawn in one font and size.
//!
//! Tests: the field that gets no class of its own is measured by Blinc in the default font while its text is drawn in
//! the font it inherits from the window (the caret then stands away from the last character); the class and its rule
//! in the style sheet are what prevent that, so they are held here. The measure on the screen (caret gap per block of
//! characters, at the level of the type components) is the typing test of `fs-files`.

use fs_components_primitives::SEARCH_FIELD_STYLES;

#[test]
fn the_body_field_has_a_rule_that_gives_it_the_font_and_the_size_it_is_drawn_in() {
    assert!(SEARCH_FIELD_STYLES.contains(".fs-body-field"));
    assert!(SEARCH_FIELD_STYLES.contains("font-family: var(--font-sans)"));
    assert!(SEARCH_FIELD_STYLES.contains("font-size: 14px"));
}

#[test]
fn counter_test_without_the_rule_the_class_would_do_nothing() {
    // The class is only a handle: with the rule removed from the sheet there is nothing that measures with the font.
    let without = SEARCH_FIELD_STYLES.replace("font-family: var(--font-sans);", "");
    assert!(
        !without.contains("font-family"),
        "the check above would be red without the rule"
    );
}
