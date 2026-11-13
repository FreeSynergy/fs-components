//! Group `render`: what `fs-components-render` draws itself: the theme variables of the style sheet and a line of text
//! in the bundled Craft font.
//!
//! Tests: picture comparison against `contract/umbau/bilder/render`.

use blinc_layout::div::div;
use fs_components_fields::Look;
use fs_components_pictures::{Group, Tolerance};
use fs_components_styles::StyleId;

fn group() -> Group {
    Group::new("render", (200, 64), Tolerance::EXACT)
}

#[test]
fn the_style_sheet_reaches_the_picture() {
    let css = ".u-box { background: var(--primary); } .u-err { background: var(--error-bg); }";
    group().check("sheet_vars", css, |_, _| {
        div().flex_row().child(div().w(100.0).h(64.0).class("u-box")).child(div().w(100.0).h(64.0).class("u-err"))
    });
}

#[test]
fn a_line_of_text_in_the_craft_font() {
    group().check("text_craft", "", |_, _| {
        let look = Look::new(StyleId::Craft, false);
        div().p(8.0).child(look.text("Craft 0123 fs", 14.0, look.palette().text))
    });
}
