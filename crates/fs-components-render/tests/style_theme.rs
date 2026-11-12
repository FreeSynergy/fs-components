//! A style as a Blinc theme: the tokens the components read come from the style.
#![allow(
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::float_cmp
)]

/// `0xRRGGBB` of a colour (alpha dropped).
fn rgb(c: Color) -> u32 {
    let b = |v: f32| (v * 255.0).round() as u32;
    (b(c.r) << 16) | (b(c.g) << 8) | b(c.b)
}

use blinc_core::Color;
use blinc_theme::Theme;
use fs_components_render::{StyleTheme, chosen_style, style_bundle, use_style};
use fs_components_styles::{StyleId, style};

#[test]
fn the_craft_theme_has_square_corners_a_white_primary_and_the_dark_ground() {
    let t = StyleTheme::new(style(StyleId::Craft), true);
    assert_eq!(t.name(), "fs-craft");
    let r = t.radii();
    assert_eq!(
        (r.radius_default, r.radius_md, r.radius_xl, r.radius_full),
        (0.0, 0.0, 0.0, 0.0)
    );
    let c = t.colors();
    assert_eq!(rgb(c.background), 0x121316);
    assert_eq!(rgb(c.primary), 0xF2F1EE);
    assert_eq!(rgb(c.input_bg), 0x060607);
    assert!(
        t.shadows().shadow_md.is_empty(),
        "a flat style has no shadow"
    );
    assert_eq!(t.typography().text_xs, 11.0);
}

#[test]
fn a_style_that_is_dark_by_nature_looks_the_same_in_a_light_system() {
    let b = style_bundle(StyleId::Craft, ".x { color: red; }");
    let light = rgb(b.light.colors().background);
    assert_eq!(light, rgb(b.dark.colors().background));
    assert!(b.css_sources.iter().any(|s| s.contains(".craft-tile")));
    assert!(b.css_sources.last().unwrap().contains(".x"));
}

#[test]
fn another_style_keeps_its_light_and_dark_theme_and_gets_no_craft_sheet() {
    let b = style_bundle(StyleId::Admin, "");
    assert_ne!(
        rgb(b.light.colors().background),
        rgb(b.dark.colors().background)
    );
    assert!(!b.css_sources.iter().any(|s| s.contains(".craft-tile")));
}

#[test]
fn the_choice_is_per_process_and_none_is_the_platform_theme() {
    assert_eq!(chosen_style(), None);
    use_style(Some(StyleId::Craft));
    assert_eq!(chosen_style(), Some(StyleId::Craft));
    use_style(None);
    assert_eq!(chosen_style(), None);
}
