//! The tolerance of the `charts` group lets font differences through and still finds a move, a wrong colour and a
//! missing part.
//!
//! Tests: counter-tests on the stored references of `contract/umbau/bilder/charts` (no drawing, no GPU).

use fs_components_pictures::{Group, Tolerance, compare};
use image::RgbaImage;

fn reference(name: &str) -> RgbaImage {
    let group = Group::new("charts", (320, 200), Tolerance::SYSTEM_FONT);
    image::open(group.reference_path(name))
        .expect("reference")
        .to_rgba8()
}

fn moved(img: &RgbaImage, dx: u32, dy: u32) -> RgbaImage {
    let (w, h) = img.dimensions();
    RgbaImage::from_fn(w, h, |x, y| {
        *img.get_pixel((x + w - dx) % w, (y + h - dy) % h)
    })
}

fn red(img: &RgbaImage) -> RgbaImage {
    let mut out = img.clone();
    out.pixels_mut()
        .for_each(|p| p.0[0] = p.0[0].saturating_add(120));
    out
}

fn inverted(img: &RgbaImage) -> RgbaImage {
    let mut out = img.clone();
    out.pixels_mut().for_each(|p| {
        for c in &mut p.0[..3] {
            *c = 255 - *c;
        }
    });
    out
}

fn blank(img: &RgbaImage) -> RgbaImage {
    RgbaImage::from_pixel(img.width(), img.height(), *img.get_pixel(0, 0))
}

#[test]
fn the_same_picture_and_a_move_of_one_pixel_pass() {
    for name in ["bars", "line"] {
        let a = reference(name);
        assert!(compare(&a, &a, Tolerance::SYSTEM_FONT).within(Tolerance::SYSTEM_FONT));
        let one = moved(&a, 1, 0);
        let r = compare(&a, &one, Tolerance::SYSTEM_FONT);
        assert!(r.within(Tolerance::SYSTEM_FONT), "{name}: {}", r.describe());
    }
}

#[test]
fn a_move_of_three_pixels_a_wrong_colour_and_a_missing_drawing_stay_red() {
    for name in ["bars", "line"] {
        let a = reference(name);
        let cases = [
            ("moved 3 px across", moved(&a, 3, 0)),
            ("moved 3 px down", moved(&a, 0, 3)),
            ("red channel +120", red(&a)),
            ("inverted", inverted(&a)),
            ("nothing drawn", blank(&a)),
        ];
        for (what, picture) in cases {
            let r = compare(&a, &picture, Tolerance::SYSTEM_FONT);
            assert!(
                !r.within(Tolerance::SYSTEM_FONT),
                "{name}: {what} must fail, got {}",
                r.describe()
            );
        }
    }
}
