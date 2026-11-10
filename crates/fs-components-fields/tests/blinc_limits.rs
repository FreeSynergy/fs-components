//! Tests: what the styles ask of Blinc, measured. A test that fails here names a limit of Blinc
//! (its git-bug entry is in the `#[ignore]` text); run them with `--ignored` to measure again.
//!
//! Pattern: none (tests). They need a graphics adapter.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use blinc_core::Color;
use blinc_layout::div::div;
use common::{draw, serial, size_of};
use fs_components_fields::Look;
use fs_components_styles::StyleId;

/// Sum over the rows of the largest step between two neighbouring pixels in a row of grey.
fn edge(img: &image::RgbaImage, x0: u32, x1: u32, y: u32) -> u32 {
    (x0..x1 - 1)
        .map(|x| u32::from(img.get_pixel(x, y).0[0].abs_diff(img.get_pixel(x + 1, y).0[0])))
        .max()
        .unwrap_or(0)
}

/// Black and white stripes, a panel of the Apple chrome surface over the right half.
fn stripes_with_panel(look: &Look) -> blinc_layout::div::Div {
    let mut row = div().flex_row().w(400.0).h(100.0);
    for i in 0..40 {
        let c = if i % 2 == 0 {
            Color::rgba(0.0, 0.0, 0.0, 1.0)
        } else {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        };
        row = row.child(div().w(10.0).h(100.0).bg(c));
    }
    let panel = look.surface(div().w(200.0).h(100.0), false);
    div()
        .w(400.0)
        .h(100.0)
        .child(row)
        .child(div().absolute().left(200.0).top(0.0).child(panel))
}

#[test]
fn the_glass_of_the_apple_style_blurs_what_is_behind_it() {
    let _g = serial();
    let look = Look::new(StyleId::Apple, false);
    let img = draw(stripes_with_panel(&look), 400, 100, "limit-glass.png");
    let (sharp, glass) = (edge(&img, 5, 195, 50), edge(&img, 205, 395, 50));
    println!("MEASURE glass: sharpest step outside {sharp}, under the panel {glass}");
    assert!(
        sharp > 200,
        "the stripes themselves must be sharp ({sharp})"
    );
    assert!(
        glass < sharp / 2,
        "glass did not blur: step {glass} against {sharp}"
    );
}

#[test]
fn the_line_height_of_text_follows_the_style() {
    let _g = serial();
    for id in StyleId::ALL {
        let look = Look::new(id, false);
        let s = look.style;
        let ui = div().flex_col().child(div().id("t").child(look.text(
            "Mg",
            s.text.body,
            look.palette().text,
        )));
        let tree = common::layout(ui, 300.0, 200.0);
        let (_, h) = size_of(&tree, "t");
        println!(
            "MEASURE line height {}: asked {}, got {h}",
            id.name(),
            s.text.body_line
        );
        assert!(
            (h - s.text.body_line).abs() < 1.0,
            "{}: asked {}, got {h}",
            id.name(),
            s.text.body_line
        );
    }
}

#[test]
#[ignore = "Blinc: Svg::stroke() does not colour the line; Svg::color() does (git-bug 7100dfad91158316ffce34a1298deae1b59d42b843437751b5ff51bf07311731 in the Blinc clone)"]
fn an_svg_stroke_colour_is_drawn() {
    let _g = serial();
    let src = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\"><path d=\"M4 12h16\" fill=\"none\" stroke=\"#00ff00\" stroke-width=\"4\"/></svg>";
    let red = Color::rgba(1.0, 0.0, 0.0, 1.0);
    let by_stroke = draw(
        blinc_layout::svg::svg(src).size(24.0, 24.0).stroke(red),
        24,
        24,
        "limit-svg-stroke.png",
    );
    let by_color = draw(
        blinc_layout::svg::svg(src).size(24.0, 24.0).color(red),
        24,
        24,
        "limit-svg-color.png",
    );
    let reddish = |img: &image::RgbaImage| {
        img.pixels()
            .filter(|p| p.0[0] > 180 && p.0[1] < 80 && p.0[2] < 80)
            .count()
    };
    println!(
        "MEASURE svg: red pixels by stroke() {}, by color() {}",
        reddish(&by_stroke),
        reddish(&by_color)
    );
    assert!(reddish(&by_stroke) > 20, "stroke() drew no red line");
}
