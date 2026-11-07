//! The size of a search field reaches the picture.
//!
//! Tests: Pixel test — skipped (with a message) where there is no graphics adapter. No window is
//! opened. The numbers depend on the theme, so only the order is checked.

use blinc_layout::div::div;
use fs_components_primitives::{FieldSize, SearchField};
use fs_components_render::{Canvas, RenderError, draw_to_png};

/// Height in pixels of what is drawn in the middle column; `None` without a graphics adapter.
fn drawn_height(size: FieldSize) -> Option<u32> {
    let dir = std::env::temp_dir().join(format!("fs-search-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{size:?}.png"));
    let field = SearchField::new("Search").size(size);
    let canvas = Canvas::new(300, 200, ".t-page { background: var(--background); }");
    let result = draw_to_png(
        &canvas,
        |w, h| page(w, h).child(field.build()),
        || {},
        &path,
    );
    let height = match result {
        Err(RenderError::NoGraphics(why)) => {
            eprintln!("skipped: {why}");
            None
        }
        other => {
            other.unwrap();
            let picture = image::open(&path).unwrap().to_rgba8();
            let page = *picture.get_pixel(299, 199);
            let rows = (0..200).filter(|y| *picture.get_pixel(150, *y) != page);
            let (first, last) = (rows.clone().min()?, rows.max()?);
            Some(last - first + 1)
        }
    };
    let _ = std::fs::remove_dir_all(&dir);
    height
}

fn page(width: f32, height: f32) -> blinc_layout::div::Div {
    div().class("t-page").w(width).h(height)
}

#[test]
fn a_small_field_is_lower_than_the_default_and_the_default_is_unchanged() {
    let Some(small) = drawn_height(FieldSize::Small) else {
        return;
    };
    let medium = drawn_height(FieldSize::Medium).unwrap();
    let large = drawn_height(FieldSize::Large).unwrap();
    assert!(small < medium, "small {small}, medium {medium}");
    assert!(medium < large, "medium {medium}, large {large}");
}
