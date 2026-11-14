//! Group `styles`: the tokens of the Craft style as colour strips and radius boxes (step `before` of the rebuild).
//!
//! Tests: picture comparison against `contract/umbau/bilder/styles`.

use blinc_layout::div::div;
use blinc_layout::prelude::ElementBuilder;
use fs_components_fields::Look;
use fs_components_pictures::{Group, Tolerance};
use fs_components_styles::StyleId;

fn group() -> Group {
    Group::new("styles", (260, 64), Tolerance::EXACT)
}

fn strip(dark: bool) -> impl ElementBuilder {
    let look = Look::new(StyleId::Craft, dark);
    let p = look.palette();
    let colours = [
        p.ground,
        p.panel,
        p.field,
        p.text,
        p.text_weak,
        p.border,
        p.accent,
        p.on_accent,
        p.focus,
        p.hover,
        p.selected,
        p.error,
        p.success,
    ];
    colours.iter().fold(div().flex_row().gap(0.0), |row, c| {
        row.child(div().w(20.0).h(64.0).bg(Look::color(*c)))
    })
}

#[test]
fn the_craft_palette_light() {
    group().check("palette_light", "", |_, _| strip(false));
}

#[test]
fn the_craft_palette_dark() {
    group().check("palette_dark", "", |_, _| strip(true));
}
