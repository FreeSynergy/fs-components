//! The comparison itself is proven here: the software GPU is the one in use, drawing twice gives no difference,
//! and a changed colour makes the check red.
//!
//! Tests: counter-tests of `fs-components-pictures`; the drawing ones skip (message) without lavapipe, except in CI.

use std::panic::{AssertUnwindSafe, catch_unwind};

use blinc_core::Color;
use blinc_layout::div::div;
use fs_components_pictures::{Group, Report, Tolerance, compare, diff_image, gpu};
use image::{Rgba, RgbaImage};

fn flat(w: u32, h: u32, c: [u8; 4]) -> RgbaImage {
    RgbaImage::from_pixel(w, h, Rgba(c))
}

fn boxed(x: u32) -> RgbaImage {
    let mut img = flat(20, 10, [0, 0, 0, 255]);
    for y in 3..7 {
        for dx in 0..1 {
            img.put_pixel(x + dx, y, Rgba([255, 255, 255, 255]));
        }
    }
    img
}

#[test]
fn equal_pictures_report_nothing_and_one_changed_pixel_is_counted() {
    let a = flat(8, 8, [10, 20, 30, 255]);
    assert_eq!(compare(&a, &a, Tolerance::EXACT).differing, 0);
    let mut b = a.clone();
    b.put_pixel(3, 4, Rgba([10, 20, 31, 255]));
    let r = compare(&a, &b, Tolerance::EXACT);
    assert_eq!((r.differing, r.max_channel, r.first), (1, 1, Some((3, 4))));
    assert!(!r.within(Tolerance::EXACT));
    let loose = Tolerance {
        channel: 1,
        ..Tolerance::EXACT
    };
    assert!(
        compare(&a, &b, loose).within(loose),
        "a channel step of 1 is allowed there"
    );
    let few = Tolerance {
        pixels: 1,
        ..Tolerance::EXACT
    };
    assert!(
        compare(&a, &b, few).within(few),
        "one pixel is allowed there"
    );
}

#[test]
fn a_move_of_two_pixels_is_allowed_only_by_the_shift() {
    let (a, b) = (boxed(5), boxed(7));
    assert!(compare(&a, &b, Tolerance::EXACT).differing > 0);
    let shift = |s| Tolerance {
        shift: s,
        ..Tolerance::EXACT
    };
    assert!(compare(&a, &b, shift(2)).within(shift(2)));
    assert!(
        compare(&a, &boxed(9), shift(2)).differing > 0,
        "a line four pixels away is too far for 2"
    );
}

#[test]
fn another_size_is_never_within_and_the_diff_picture_marks_the_pixel() {
    let r: Report = compare(
        &flat(4, 4, [0; 4]),
        &flat(5, 4, [0; 4]),
        Tolerance {
            pixels: usize::MAX - 1,
            ..Tolerance::EXACT
        },
    );
    assert!(
        r.size_mismatch.is_some()
            && !r.within(Tolerance {
                pixels: usize::MAX,
                ..Tolerance::EXACT
            })
    );
    let a = flat(4, 4, [9; 4]);
    let mut b = a.clone();
    b.put_pixel(1, 2, Rgba([200, 9, 9, 255]));
    assert_eq!(
        diff_image(&a, &b, Tolerance::EXACT).get_pixel(1, 2).0,
        [255, 0, 0, 255]
    );
}

#[test]
fn the_adapter_in_use_is_the_software_one() {
    match gpu::force_software() {
        Ok(name) => {
            eprintln!("adapter: {name}");
            assert!(name.to_lowercase().contains("llvmpipe"), "{name}");
        }
        Err(why) => {
            assert!(std::env::var_os("CI").is_none(), "{why}");
            eprintln!("skipped: {why}");
        }
    }
}

fn swatch(c: [u8; 3]) -> impl FnOnce(f32, f32) -> blinc_layout::div::Div {
    move |w, h| {
        div().w(w).h(h).bg(Color::rgb(
            f32::from(c[0]) / 255.0,
            f32::from(c[1]) / 255.0,
            f32::from(c[2]) / 255.0,
        ))
    }
}

#[test]
fn drawing_twice_gives_no_difference() {
    let g = Group::new("selftest", (64, 32), Tolerance::EXACT);
    let draw = || g.draw("", swatch([51, 102, 153]));
    let (Ok(a), Ok(b)) = (draw(), draw()) else {
        assert!(std::env::var_os("CI").is_none());
        return eprintln!("skipped: no software GPU");
    };
    let r = compare(&a, &b, Tolerance::EXACT);
    assert_eq!((r.differing, r.max_channel), (0, 0), "{}", r.describe());
}

#[test]
fn a_changed_colour_is_seen_and_the_check_is_red_with_a_diff_picture() {
    let g = Group::new("selftest", (64, 32), Tolerance::EXACT);
    let (Ok(a), Ok(b)) = (
        g.draw("", swatch([51, 102, 153])),
        g.draw("", swatch([51, 102, 154])),
    ) else {
        assert!(std::env::var_os("CI").is_none());
        return eprintln!("skipped: no software GPU");
    };
    let r = compare(&a, &b, Tolerance::EXACT);
    assert!(
        r.differing > 0,
        "one blue step must be seen: {}",
        r.describe()
    );
    // the reference of the group is `a`, the new picture is `b`: `judge` must fail and leave a diff picture
    g.write_reference("swatch", &a);
    let red = catch_unwind(AssertUnwindSafe(|| g.judge("swatch", &b)));
    let _ = std::fs::remove_dir_all(Group::root().join("selftest"));
    assert!(red.is_err(), "a changed colour must make the check fail");
}
