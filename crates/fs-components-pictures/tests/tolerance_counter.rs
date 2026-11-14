//! The tolerances of the blocks with text let font differences through and still finds a move, a wrong colour and a
//! missing part.
//!
//! Tests: counter-tests on the stored references of `contract/umbau/bilder/charts` (no drawing, no GPU).

use fs_components_pictures::{Group, Tolerance, compare};
use image::RgbaImage;

/// Every block that has a tolerance: (group, block, tolerance), the same as in `tests/*_group.rs`.
const BLOCKS: [(&str, &str, Tolerance); 10] = [
    ("charts", "bars", Tolerance::SYSTEM_FONT),
    ("charts", "line", Tolerance::SYSTEM_FONT),
    ("kanban", "board_moves", Tolerance::SYSTEM_FONT_BOARD),
    ("kanban", "board_plain", Tolerance::SYSTEM_FONT_BOARD),
    ("primitives", "search_medium", Tolerance::SYSTEM_FONT_SMALL),
    (
        "primitives",
        "search_small_body",
        Tolerance::SYSTEM_FONT_SMALL,
    ),
    ("render", "text_craft", Tolerance::RASTER),
    ("render", "sheet_vars", Tolerance::RASTER),
    ("styles", "palette_light", Tolerance::RASTER),
    ("styles", "palette_dark", Tolerance::RASTER),
];

fn reference(group: &str, name: &str) -> RgbaImage {
    let group = Group::new(
        match group {
            "charts" => "charts",
            "kanban" => "kanban",
            "primitives" => "primitives",
            "styles" => "styles",
            _ => "render",
        },
        (1, 1),
        Tolerance::EXACT,
    );
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
fn the_same_picture_passes_and_a_move_of_one_pixel_only_where_the_tolerance_allows_it() {
    for (group, name, t) in BLOCKS {
        let a = reference(group, name);
        assert!(compare(&a, &a, t).within(t), "{group}/{name}");
        let r = compare(&a, &moved(&a, 1, 0), t);
        // Where the tolerance has a shift, a move of 1 px is allowed; where it has none (RASTER), it stays red.
        assert_eq!(
            r.within(t),
            t.shift >= 1,
            "{group}/{name}: {}",
            r.describe()
        );
    }
}

#[test]
fn a_move_of_three_pixels_a_wrong_colour_and_a_missing_drawing_stay_red() {
    for (group, name, t) in BLOCKS {
        let a = reference(group, name);
        let cases = [
            ("moved 3 px across", moved(&a, 3, 0)),
            ("moved 3 px down", moved(&a, 0, 3)),
            ("red channel +120", red(&a)),
            ("inverted", inverted(&a)),
            ("nothing drawn", blank(&a)),
        ];
        for (what, picture) in cases {
            if picture == a {
                continue; // a vertical move of vertical strips changes nothing: there is nothing to find
            }
            let r = compare(&a, &picture, t);
            assert!(
                !r.within(t),
                "{group}/{name}: {what} must fail, got {}",
                r.describe()
            );
        }
    }
}
